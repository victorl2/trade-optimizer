fn main(){
    println!("Hello World");
    println!("{}", mars_calculator(92.0));
}   

fn mars_calculator(kg_earth_weight: f64) -> f64 {
    let earth_gravity: f64 = 9.81;
    let mars_gravity = 3.711;
    let mars_weight = kg_earth_weight * earth_gravity / mars_gravity;
    mars_weight
}