use crate::print;
use crate::println;

use crate::setup::Serial;

pub fn print_normalize(median: f32, iqr: f32, input: &mut [f32]) {
    /*
     * Normalize the input array. This is done by subtracting the median from each value and then dividing by the interquartile range.
     * This is done because for some reason this is the best for accuracy on an older test data set, should recompile a new dataset and verify that this is true.
     * Note that the row is modified in place.
     */

    normalize(input, median, iqr);

    print!("Normalized: ");
    for (i, &value) in input.iter().enumerate() {
        if i < input.len() - 1 {
            print!("{value},");
        } else {
            println!("{value}");
        }
    }

    let _ = Serial::flush();
}

pub fn normalize(row: &mut [f32], median: f32, iqr: f32) {
    for item in row.iter_mut() {
        // Iterate over the input slice which gives us a mutable iterator over each element
        *item = (*item - median) / iqr;
    }
}
