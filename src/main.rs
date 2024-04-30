#[cfg(test)]
mod tests;

pub fn example_uncovered_function() -> u32 {
    println!("bye!");

    4
}

pub fn example_covered_function() -> u32 {
    println!("hey!");

    3
}

fn main() {
    println!("Hello, world!");
}
