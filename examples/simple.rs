use adage::prelude::*;

#[task]
async fn get_email() -> String {
    format!("{}@looney.com", "bugs")
}

#[task]
async fn get_name() -> (String, String) {
    let first = "Bugs".to_string();
    let last = "Bunny".to_string();
    (first, last)
}

#[task]
async fn send_email(email: String, name: (String, String)) {
    let (first, last) = name;
    println!("Sending email to {}", email);
    println!("To: {} {}, Hello!", first, last);
}

//#[adage::flow]
//fn my_flow() {
//    let email = get_email();
//    let name = get_name();
//    send_email(email, name);
//}
fn my_flow(ex: &mut impl Executor) {
    let email = get_email().submit(ex);
    let name = get_name().submit(ex);
    send_email(email, name).submit(ex);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let mut ex = BasicExecutor::new();
    my_flow(&mut ex);
    ex.run().await;
}
