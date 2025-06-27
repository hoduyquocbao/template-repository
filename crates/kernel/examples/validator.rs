use kernel::validator::{System, Text, Number};

fn main() {
    let validator = System::new();
    // Kiểm tra text
    let result = validator.text("hello", &[Text::Min(3), Text::Max(10)]);
    println!("Text valid: {}", result.is_ok());
    let result = validator.text("hi", &[Text::Min(3)]);
    println!("Text valid (should fail): {}", result.is_ok());
    // Kiểm tra number
    let result = validator.number(&5.0, &[Number::Min(1.0), Number::Max(10.0)]);
    println!("Number valid: {}", result.is_ok());
    let result = validator.number(&0.0, &[Number::Positive]);
    println!("Number valid (should fail): {}", result.is_ok());
} 