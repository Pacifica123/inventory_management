use rand::Rng;
use rand_distr::{Distribution, Normal};

// внешние факторы
const COST: f64 = 10.0; // цена за хранение на складе на единицу продукции в рублях
const PRODUCT_COST: f64 = 100.0; // себестоимость единицы продукции в рублях

#[derive(Debug)]
struct Params {
    start_storage: u32, // начальный объем готовой продукции на складе, в шт.
    max_storage: u32, // максимальный допустимый объем продукции на складе, в шт.
    period_len: u32, // количество периодов для планирования запасов
    power: f64, // мощность производства (мат. ожидание количества производства за период)
    factor: f64, // погрешность производства
    mean_sales: f64, // среднее количество продаж за период, в шт.
    mean_price: f64, // рыночная цена за единицу продукции, в рублях
    sigma_price: f64, // разброс цены за единицу продукции
    sigma_sales: f64, // разброс количества продаж за период
}

#[derive(Debug)]
struct Output {
    revenue: f64, // доход от продажи продукции за период, в рублях
    general_loss: f64, // убытки от недопроизводства или затраты на хранение, в рублях
    deltas: Vec<i32>, // <0 - недопроизводство, >0 - перепроизводство, в шт.
    mean: f64, // среднее значение deltas за всё время
}

#[derive(Debug)]
struct InventoryManager {
    params: Params,
    storage: u32, // количество готовой продукции на складе, в шт.
    production: u32, // количество производства за период, в шт.
    price: f64, // цена за единицу продукции за период, в рублях
    sales: u32, // количество успешных продаж за период, в шт.
    revenue: f64, // доход от продажи продукции за период, в рублях
    loss: f64, // убытки за период, в рублях
    out: Output,
}

impl InventoryManager {
    fn new(params: Params, storage: u32) -> InventoryManager {
        if storage > params.max_storage {
            panic!("Недопустимое значение storage");
        }
        if params.factor >= params.power || params.factor <= 0.0 {
            panic!("Недопустимое значение factor");
        }
        InventoryManager {
            params,
            storage,
            production: 0,
            price: 0.0,
            sales: 0,
            revenue: 0.0,
            loss: 0.0,
            out: Output { revenue: 0.0, general_loss: 0.0, deltas: vec![], mean: 0.0 },
        }
    }

    fn gen_normal_price(&mut self) {
        let normal_dist = Normal::new(self.params.mean_price, self.params.sigma_price).unwrap();
        self.price = normal_dist.sample(&mut rand::thread_rng());
    }

    fn gen_normal_sales(&mut self) {
        let normal_dist = Normal::new(self.params.mean_sales, self.params.sigma_sales).unwrap();
        self.sales = normal_dist.sample(&mut rand::thread_rng()).round() as u32;
    }

    fn gen_production(&mut self) {
        let delta_production = rand::thread_rng().gen_range(-self.params.factor..=self.params.factor);
        self.production = (self.params.power + delta_production).round() as u32;
    }

    fn modeling_iteration(&mut self) {
        self.gen_normal_price();
        self.gen_normal_sales();
        self.gen_production();

        if self.storage + self.production <= self.params.max_storage {
            self.storage += self.production;
        } else {
            let product_loss = self.production - (self.params.max_storage - self.storage);
            self.storage = self.params.max_storage;
            self.loss += product_loss as f64 * PRODUCT_COST;
            self.out.general_loss += product_loss as f64;
        }

        self.calc_revenue();
    }

    fn calc_revenue(&mut self) {
        let delta = self.storage as i32 - self.sales as i32;
        self.out.deltas.push(delta);

        if self.storage > self.sales {
            self.storage -= self.sales;
            self.out.revenue += self.sales as f64 * self.price;
        } else {
            self.out.revenue += self.storage as f64 * self.price;
            self.storage = 0;
        }

        if delta > 0 {
            self.out.general_loss += delta as f64 * COST; // затраты на хранение
        } else if delta < 0 {
            self.out.general_loss += delta.abs() as f64 * self.price; // упущенная продажа
        }
    }

    fn calc_mean(&mut self) {
        if !self.out.deltas.is_empty() {
            self.out.mean = self.out.deltas.iter().copied().sum::<i32>() as f64 / self.out.deltas.len() as f64;
        }
    }

    fn modeling_cycle(&mut self, n: u32) {
        for _ in 0..n {
            self.reset_output();
            for _ in 0..self.params.period_len {
                self.modeling_iteration();
            }
            self.calc_mean();
            self.write_output();
        }
    }

    fn reset_output(&mut self) {
        self.out.deltas.clear();
        self.out.mean = 0.0;
        self.out.revenue = 0.0;
        self.out.general_loss = 0.0;
        self.loss = 0.0;
        self.storage = self.params.start_storage;
        self.production = 0;
        self.price = 0.0;
        self.sales = 0;
        self.revenue = 0.0;
    }

    fn write_output(&self) {
        println!("ИТОГО:");
        println!("Прибыль: {}", self.out.revenue);
        println!("Общие потери: {}", self.out.general_loss);
        println!("Среднее: {}", self.out.mean);
        println!("На складе: {:?}", self.out.deltas);
    }

    fn debug_output(&self) {
        println!("Выручка: {} руб", self.out.revenue);
        println!("Общие потери: {} руб", self.out.general_loss);
        println!("Среднее на складе: {} шт", self.out.mean);
        println!("На складе: {:?} ", self.out.deltas);
        println!("Сейчас на складе: {} ", self.storage);
        println!("Производство: {} ", self.production);
        println!("Текущая цена: {} ", self.price);
        println!("Текущие продажи: {} ", self.sales);
        println!("\n----------------------------------------");
    }
}

fn main() {
    let params = Params {
        start_storage: 0,
        max_storage: 1000,
        power: 120.0,
        factor: 2.0,
        period_len: 10,
        mean_sales: 100.0,
        mean_price: 100.0,
        sigma_price: 1.0,
        sigma_sales: 1.0,
    };

    let mut inventory = InventoryManager::new(params, 1000);
    inventory.modeling_cycle(100);
}
