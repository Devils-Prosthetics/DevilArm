use crate::setup::Serial;

pub fn print_normalize(median: f32, iqr: f32, input: &mut [f32]) {
    /*
     * Normalize the input array. This is done by subtracting the median from each value and then dividing by the interquartile range.
     * This is done because for some reason this is the best for accuracy on an older test data set, should recompile a new dataset and verify that this is true.
     * Note that the row is modified in place.
     */

    normalize(input, median, iqr);

    let _ = Serial::write(b"Normalized: ");
    for (i, &value) in input.iter().enumerate() {
        let mut buffer = dtoa::Buffer::new();
        let value = buffer.format(value);
        let value = value.as_bytes();
        if i < input.len() - 1 {
            let _ = Serial::write(value);
            let _ = Serial::write(b",");
        } else {
            let _ = Serial::write(value);
            let _ = Serial::write(b"\n");
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
