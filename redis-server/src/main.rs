mod serde;

fn main() {
    println!("{}", serde::deserialize("$-1\r\n").unwrap());
    println!("{}", serde::deserialize("*1\r\n$4\r\nping\r\n").unwrap());
    println!("{}", serde::deserialize(":666\r\n").unwrap());
    println!("{}", serde::deserialize(":-1000\r\n").unwrap());
    println!("{}", serde::deserialize("*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n").unwrap());
    println!("{}", serde::deserialize("*2\r\n$3\r\nget\r\n$3\r\nkey\r\n").unwrap());
    println!("{}", serde::deserialize("+OK\r\n").unwrap());
    println!("{}", serde::deserialize("-Error message\r\n").unwrap());

    println!("{}", serde::serialize(serde::Response::Null));
    println!("{}", serde::serialize(serde::Response::Array(["ping".into()].into())));
    println!("{}", serde::serialize(serde::Response::Integer(666)));
    println!("{}", serde::serialize(serde::Response::Integer(-1000)));
    println!("{}", serde::serialize(serde::Response::Array(["echo".into(), "hello world".into()].into())));
    println!("{}", serde::serialize(serde::Response::SimpleString("OK".into())));
    println!("{}", serde::serialize(serde::Response::Array(vec!["get".into(), "key".into()])));
    println!("{}", serde::serialize(serde::Response::Error("Error message".into())));

    println!("{:?}", serde::serialize(serde::Response::Null));
    println!("{:?}", serde::serialize(serde::Response::Array(["ping".into()].into())));
    println!("{:?}", serde::serialize(serde::Response::Integer(666)));
    println!("{:?}", serde::serialize(serde::Response::Integer(-1000)));
    println!("{:?}", serde::serialize(serde::Response::Array(["echo".into(), "hello world".into()].into())));
    println!("{:?}", serde::serialize(serde::Response::SimpleString("OK".into())));
    println!("{:?}", serde::serialize(serde::Response::Array(vec!["get".into(), "key".into()])));
    println!("{:?}", serde::serialize(serde::Response::Error("Error message".into())));
}
