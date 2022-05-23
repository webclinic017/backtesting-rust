#[test]
fn general_test() {
    use bytemuck;
    use rand::prelude::*;

    // let mut rng = rand::thread_rng();
    // let v: Vec<f32> = (0..1000).map(|_| rng.gen::<f32>()*100.0).collect();
    let u: Vec<f32> = vec![f32::NAN; 4];
    let v: &[u8] = bytemuck::cast_slice(u.as_slice());

    let l_u = u.len();
    let l_v = v.len();

    let r = std::mem::size_of::<u8>();

    let x = v.len() / std::mem::size_of::<f32>();

    println!("{x}");

}