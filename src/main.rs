pub mod normal;
use rand::Rng;

// внешние факторы
const COST: f64 = 10.0; //цена за хранение на складе на единицу продукции в рублях
const PRODUCT_COST: f64 = 100.0; //себестоимость единицы продукции в рублях
const MEAN_SALES: f64 = 100.0; //среднее количество продаж за период в шт.
const MEAN_PRICE: f64 = 100.0; //рыночная цена за продажу на единицу продукции в рублях
const SIGMA_PRICE: f64 = 1.0; //разброс цены за продажу на единицу продукции в рублях (нестабильность рынка)
const SIGMA_SALES: f64 = 1.0; //разброс количества продаж за период в шт. (нестабильность рынка)

struct Params {
    start_storage: u32, //начальный объем готовой продукции хранящейся на складе, в шт.
    max_storage: u32, //максимальный допустимый объем готовой продукции хранящейся на складе, в шт.
    period_len: u32, //количество периодов на которые ведется планирование управления запасами и прогнозирование цен
    power: f64, //мощность производства (мат. ожидание количества производства за период)
    // cost: f64, //цена за хранение на складе на единицу продукции
    factor: f64, //погрешность производства
}

struct Output {
    revenue: f64, //доход от продажи готовой продукции на складе за период, в рублях
    General_loss: f64, //убытки от недопроизводства или затраты на хранение на складе от перепроизводства за всё время, в рублях
    deltas: Vec<i32>, //<0 - недопроизводство, >0 - перепроизводство, в шт.
    mean: f64, //среднее значение deltas за всё время; <0 - недопроизводство, >0 - перепроизводство, в шт.
}

struct InventoryManager {
    params: Params,

    storage: u32, //количество готовой продукции на складе, в шт.
    production: u32, //количество производства за период, в шт.
    price: f64, //цена за продажу на единицу продукции за период, в рублях
    sales: u32, // количество успешных продаж за период, в шт.
    revenue: f64, //доход от продажи готовой продукции на складе за период, в рублях
    loss: f64, //убытки от недопроизводства или затраты на хранение на складе от перепроизводства за период, в рублях

    out: Output,
}
impl InventoryManager {
    fn new(params: Params, storage: u32) -> InventoryManager {
        if (storage > params.max_storage) {
            panic!("Недопустимое значение storage");
        }
        if (params.factor >= params.power) || (params.factor <= 0.0) {
            panic!("Недопустимое значение factor");
        }
        InventoryManager {
            params: params,
            storage: storage,
            production: 0,
            price: 0.0,
            sales: 0,
            revenue: 0.0,
            loss: 0.0,
            out: Output { revenue: 0.0, General_loss: 0.0, deltas: vec![], mean: 0.0 },
        }
    }

    fn gen_normal_price(&mut self) {
        self.price = normal::generate_standart_random(normal::Params {miu: MEAN_PRICE, sigma: SIGMA_PRICE});
    }
    fn gen_normal_sales(&mut self) {
        self.sales = normal::generate_standart_random(normal::Params {miu: MEAN_SALES, sigma: SIGMA_SALES}).floor() as u32;
    }
    fn gen_production( &mut self) {
        let delta_production = rand::thread_rng().gen_range(-self.params.factor..self.params.factor);
        self.production = (self.params.power + delta_production).floor() as u32;
    }

    fn modeling_iteration(&mut self) {
        // self.debug_output();

        self.gen_normal_price();
        self.gen_normal_sales();
        self.gen_production();

        if self.storage+self.production < self.params.max_storage {
            self.storage += self.production;
        }
        else {
            // let sum = self.storage+self.production;
            let product_loss = self.production - (self.params.max_storage - self.storage); 
            self.storage = self.params.max_storage;
            // так как мы уже никогда не продадим, то это уупущенная выгода:
            self.loss += product_loss as f64 * PRODUCT_COST;
            self.out.General_loss += product_loss as f64;
        }
        
        // self.calc_revenue();


    }

    fn calc_revenue(&mut self) {
        if self.storage > self.sales {
            // нормальный случай, когда на складе хватает продукции
            self.storage -= self.sales;
            self.out.revenue += self.sales as f64 * self.price;
        }
        else {
            // случай, когда на складе не хватает продукции
            // (продаем сколько есть)
            self.out.revenue += self.storage as f64 * self.price;
            self.storage = 0;
        }
        let delta = self.storage as i32 - self.sales as i32;
        self.out.deltas.push(delta);

        if delta > 0 {self.out.General_loss += delta as f64 * COST;} // затраты на хранение
        else if delta < 0 {self.out.General_loss += delta.abs() as f64 * self.price;} // упущенная продажа
    }

    fn calc_mean(&mut self) {
        self.out.mean = self.out.deltas.iter().sum::<i32>() as f64 / self.out.deltas.len() as f64;
    }

    fn modeling_cycle(&mut self, n: u32) {
        
        for _ in 0..n {
            self.reset_output();
            for _ in 0..self.params.period_len {
                self.modeling_iteration();
                self.calc_mean();
                self.calc_revenue();
            }
            self.write_output();
        }
    }

    fn reset_output(&mut self) {
        self.out.deltas = vec![];
        self.out.mean = 0.0;
        self.out.revenue = 0.0;
        self.out.General_loss = 0.0;
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
        println!("Общие потери: {}", self.out.General_loss);
        println!("Среднее: {}", self.out.mean);
        println!("На складе: {:?}", self.out.deltas);
    }

    fn debug_output(&self) {
        println!("Выручка: {} руб", self.out.revenue);
        println!("Общие потери: {} руб", self.out.General_loss);
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
    let mut params = Params {
        start_storage: 0,
        max_storage: 1000,
        power: 120.0,
        factor: 2.0,
        period_len: 10,
        // cost: 0.0

    };
    let mut inventory = InventoryManager::new(params, 1000);
    inventory.modeling_cycle(100);
}
