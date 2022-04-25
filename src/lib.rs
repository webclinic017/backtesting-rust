pub mod vector_utils;
pub mod utils;
pub mod strategy;


#[test]
#[allow(arithmetic_overflow)]
fn playground_test() {
    // let v = [1,0,0,0,1,-1,0,0,-1,0,0,1,-1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,-1,0,0,0,0,1];


    let x:usize = 4_usize;
    let y:usize = 7_usize;

    let r = x.wrapping_sub(y);
    println!("{}", r);

}
