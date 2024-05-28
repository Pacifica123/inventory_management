use rand::Rng;

pub (crate)
struct Params {
    pub(crate) miu : f64, // коэффициент сдвига (ср.знач)
    pub(crate) sigma : f64, // коэффицент масштаба (дисперсия)
}


fn gen_white_noise(params: &Params) -> f64{
    let mut rng = rand::thread_rng();
    let mut sum = 0.0;

    for _ in 0..12 {
        sum += rng.gen_range(0.0..1.0);
        
    }

    sum -= 6.0;

    sum
}

pub (crate)
fn generate_standart_random(params: Params) -> f64 {
    let mut sample = 0.0;

    let sample = params.miu + gen_white_noise(&params) * params.sigma;

    sample
}