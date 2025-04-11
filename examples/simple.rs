#[adage::task]
async fn get_email() -> String {
    format!("{}@looney.com", "bugs")
}

#[adage::task]
async fn get_name() -> (String, String) {
    let first = "Bugs".to_string();
    let last = "Bunny".to_string();
    (first, last)
}

#[adage::task]
async fn send_email(email: String, name: (String, String)) {
    let (first, last) = name;
    println!("Sending email to {}", email);
    println!("To: {} {}, Hello!", first, last);
}

//fn my_flow() {
//    let email = get_email();
//    let name = get_name();
//    send_email(email, name);
//}
async fn my_flow() {
    let email = get_email();
    let name = get_name();
    let send = send_email(&email, &name);

    let email = email.start();
    let name = name.start();
    let send = send.start();

    email.join().await;
    name.join().await;
    send.join().await;
}

#[tokio::main]
async fn main() {
    my_flow().await;
}
