// Prints all points in a 3d lattice - For gourd examples.

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let x: u128 = args[1].parse().expect("Input not an integer");
    let y: u128 = args[2].parse().expect("Input not an integer");
    let z: u128 = args[3].parse().expect("Input not an integer");

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                println!("({}, {}, {})", i, j, k);
            }
        }
    }
}
