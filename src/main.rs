use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct Params {
    start_storage: u32,
    max_storage: u32,
    period_len: u32,
    power: f64,
    factor: f64,
    mean_sales: f64,
    mean_price: f64,
    sigma_price: f64,
    sigma_sales: f64,
    cost: f64,
    product_cost: f64,
}

#[derive(Debug)]
struct Output {
    revenue: f64,
    general_loss: f64,
    deltas: Vec<i32>,
    mean: f64,
}

#[derive(Debug)]
struct InventoryManager {
    params: Params,
    storage: u32,
    production: u32,
    price: f64,
    sales: u32,
    revenue: f64,
    loss: f64,
    out: Output,
}

impl InventoryManager {
    fn new(params: Params, storage: u32) -> Result<InventoryManager, String> {
        if storage > params.max_storage {
            return Err(String::from("Недопустимое значение storage"));
        }
        if params.factor >= params.power || params.factor <= 0.0 {
            return Err(String::from("Недопустимое значение factor"));
        }
        Ok(InventoryManager {
            params,
            storage,
            production: 0,
            price: 0.0,
            sales: 0,
            revenue: 0.0,
            loss: 0.0,
            out: Output {
                revenue: 0.0,
                general_loss: 0.0,
                deltas: vec![],
                mean: 0.0,
            },
        })
    }

    fn gen_normal_price(&mut self, rng: &mut rand::rngs::ThreadRng) {
        let normal_dist = Normal::new(self.params.mean_price, self.params.sigma_price).unwrap();
        self.price = normal_dist.sample(rng);
    }

    fn gen_normal_sales(&mut self, rng: &mut rand::rngs::ThreadRng) {
        let normal_dist = Normal::new(self.params.mean_sales, self.params.sigma_sales).unwrap();
        self.sales = normal_dist.sample(rng).round().max(0.0) as u32; // Продажи не могут быть отрицательными
    }

    fn gen_production(&mut self, rng: &mut rand::rngs::ThreadRng) {
        let delta_production = rng.gen_range(-self.params.factor..=self.params.factor);
        self.production = (self.params.power + delta_production).round().max(0.0) as u32; // Производство не может быть отрицательным
    }

    fn modeling_iteration(&mut self, rng: &mut rand::rngs::ThreadRng) {
        self.gen_normal_price(rng);
        self.gen_normal_sales(rng);
        self.gen_production(rng);

        if self.storage + self.production <= self.params.max_storage {
            self.storage += self.production;
        } else {
            let product_loss = self.production as i32 - (self.params.max_storage as i32 - self.storage as i32).max(0);
            self.storage = self.params.max_storage;
            self.loss += product_loss as f64 * self.params.product_cost;
            self.out.general_loss += product_loss as f64;
        }

        self.calc_revenue();
    }

    fn update_storage(&mut self) {
        if self.storage > self.sales {
            self.storage -= self.sales;
        } else {
            self.storage = 0;
        }
    }

    fn calc_delta(&mut self) {
        let delta = self.storage as i32 - self.sales as i32;
        self.out.deltas.push(delta);
        if delta > 0 {
            self.out.general_loss += delta as f64 * self.params.cost; // Перепроизводство
        } else if delta < 0 {
            self.out.general_loss += delta.abs() as f64 * self.params.product_cost; // Недопроизводство
        }
    }

    fn calc_revenue(&mut self) {
        self.update_storage();
        self.out.revenue += self.sales as f64 * self.price;
        self.calc_delta();
    }

    fn calc_mean(&mut self) {
        if !self.out.deltas.is_empty() {
            self.out.mean = self.out.deltas.iter().copied().sum::<i32>() as f64 / self.out.deltas.len() as f64;
        }
    }

    fn modeling_cycle(&mut self, n: u32) -> io::Result<()> {
        let mut rng = rand::thread_rng();
        let mut file = File::create("output.txt")?;
        let mut total_revenue = 0.0;
        let mut total_loss = 0.0;
        let mut total_mean = 0.0;

        for cycle in 0..n {
            self.reset_output();
            for _ in 0..self.params.period_len {
                self.modeling_iteration(&mut rng);
            }
            self.calc_mean();
            let profit = self.out.revenue - self.out.general_loss;
            writeln!(
                file,
                "Цикл №{}: Общая прибыль: {:.2}, Общие затраты: {:.2}, Разница: {:.2}, Среднее на складе: {:.2}",
                cycle + 1,
                self.out.revenue,
                self.out.general_loss,
                profit,
                self.out.mean
            )?;

            total_revenue += self.out.revenue;
            total_loss += self.out.general_loss;
            total_mean += self.out.mean;
        }

        // Вычисляем суперсреднее
        let super_mean_revenue = total_revenue / n as f64;
        let super_mean_loss = total_loss / n as f64;
        let super_mean = total_mean / n as f64;

        writeln!(
            file,
            "Суперсреднее за {} циклов: Общая прибыль: {:.2}, Общие затраты: {:.2}, Среднее на складе: {:.2}",
            n,
            super_mean_revenue,
            super_mean_loss,
            super_mean
        )?;

        Ok(())
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
}

fn main() {
    let params = Params {
        start_storage: 0,
        max_storage: 1000,
        power: 119.0,
        factor: 2.0,
        period_len: 10,
        mean_sales: 100.0,
        mean_price: 100.0,
        sigma_price: 1.0,
        sigma_sales: 1.0,
        cost: 10.0, 
        product_cost: 100.0, 
    };

    let mut inventory = InventoryManager::new(params, 1000).expect("Ошибка при создании InventoryManager");
    inventory.modeling_cycle(1000).expect("Ошибка при записи в файл");
}
