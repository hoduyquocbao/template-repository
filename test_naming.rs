// Trường hợp VI PHẠM (phải bị chặn commit)
struct UserProfile {}
trait RenderAble {}
enum HttpRequestData {}
union MyUnionType {}
type MyTypeAlias = i32;

pub fn parseInputData() {}
let userName = "abc";
let mut parseInputData = 1;

pub fn parse_input_data() {}
let user_name = 1;
let mut parse_input_data = 2;

// Trường hợp HỢP LỆ (không bị chặn commit)
struct User {}
trait Renderable {}
enum Request {}
union Value {}
type Alias = i32;

pub fn parse() {}
let name = "abc";
let mut value = 1;

// Sử dụng API bên thứ 3 (KHÔNG bị bắt)
fn main() {
    let x = serde_json::from_str::<HttpRequestData>("{}");
    let y = external_api::parseInputData();
    let z = external_api::parse_input_data();
} 