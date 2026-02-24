use ort::Value;
use ndarray::Array4;

fn check() {
    let input = Array4::<f32>::zeros((1, 3, 224, 224));
    let t = Value::from_array(input).unwrap();
}
