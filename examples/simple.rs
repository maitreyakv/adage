use adage::{ctx, key};

struct User {
    name: String,
    email: String,
}
struct Message(String);

#[adage::provides(Vec<User>)]
#[adage::for_key(())]
#[adage::requires_context]
struct GetUsers;

#[adage::provides_for(GetUsers)]
fn get_users() {
    vec![
        User {
            name: "Bugs Bunny".to_string(),
            email: "bugs@looney.com".to_string(),
        },
        User {
            name: "Daffy Duck".to_string(),
            email: "daffy@looney.com".to_string(),
        },
    ]
}

#[adage::provides(Message)]
#[adage::for_key(User)]
#[adage::requires_context]
struct Greet {
    greeting: String,
}

#[adage::provides_for(Greet)]
fn greet() {
    Message(format!("{} {}!", self.greeting, key!().name))
}

#[adage::provides(())]
#[adage::for_key(())]
#[adage::requires_context(
    <Vec<User>>,
    <Message, User>,
)]
struct SendEmails;

#[adage::provides_for(SendEmails)]
fn send_emails() {
    let users = ctx!();
    for user in users {
        let greeting = ctx![&user];
        println!("Sending email to {}: {}", user.email, greeting.0)
    }
}

fn main() {}
