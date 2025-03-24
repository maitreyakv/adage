use adage::{Context, Layer};

struct User {
    name: String,
    email: String,
}
type Users = Vec<User>;
struct Message(String);

#[adage::requires_context]
struct GetUsers;
impl<C> Layer<C> for GetUsers
where
    C: _GetUsersContext,
{
    type Resource = Users;
    type Key = ();

    fn provide(&self, _key: &Self::Key, _ctx: C) -> Self::Resource {
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
}

#[adage::requires_context]
struct Greet {
    greeting: String,
}
impl<C> Layer<C> for Greet
where
    C: _GreetContext,
{
    type Resource = Message;
    type Key = User;

    fn provide(&self, user: &Self::Key, _ctx: C) -> Self::Resource {
        Message(format!("{} {}!", self.greeting, user.name))
    }
}

#[adage::requires_context(
    <Users>,
    <Message, User>,
)]
struct SendEmails;
impl<C> Layer<C> for SendEmails
where
    C: _SendEmailsContext,
{
    type Resource = ();
    type Key = ();

    fn provide(&self, _key: &Self::Key, ctx: C) -> Self::Resource {
        let users: Users = ctx.get(());
        for user in users {
            let greeting: Message = ctx.get(&user);
            println!("Sending email to {}: {}", user.email, greeting.0)
        }
    }
}

fn main() {}
