//#[adage::task]
//async fn get_email() -> String {
//    format!("{}@looney.com", "bugs")
//}
struct GetEmailTaskFn;
impl ::adage::TaskFn for GetEmailTaskFn {
    type Input = ();
    type Output = String;
    type Error = ::std::convert::Infallible;
    async fn run(_input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(format!("{}@looney.com", "bugs"))
    }
}
fn get_email() -> ::adage::PlannedTask<::adage::EmptyReceiver, GetEmailTaskFn> {
    ::adage::PlannedTask::new(::adage::EmptyReceiver)
}

//#[adage::task]
//async fn get_name() -> (String, String) {
//    let first = "Bugs".to_string();
//    let last = "Bunny".to_string();
//    (first, last)
//}
struct GetNameTaskFn;
impl ::adage::TaskFn for GetNameTaskFn {
    type Input = ();
    type Output = (String, String);
    type Error = ::std::convert::Infallible;
    async fn run(_input: Self::Input) -> Result<Self::Output, Self::Error> {
        let first = "Bugs".to_string();
        let last = "Bunny".to_string();
        Ok((first, last))
    }
}
fn get_name() -> ::adage::PlannedTask<::adage::EmptyReceiver, GetNameTaskFn> {
    ::adage::PlannedTask::new(::adage::EmptyReceiver)
}

//#[adage::task]
//async fn send_email(email: String, name: (String, String)) {
//    let (first, last) = name;
//    println!("Sending email to {}", email);
//    println!("To: {} {}, Hello!", first, last);
//}
struct SendEmailTaskFnInput {
    email: String,
    name: (String, String),
}
struct SendEmailTaskFnInputReceiver {
    email: ::tokio::sync::broadcast::Receiver<String>,
    name: ::tokio::sync::broadcast::Receiver<(String, String)>,
}
impl ::adage::InputReceiver for SendEmailTaskFnInputReceiver {
    type Data = SendEmailTaskFnInput;
    type Error = ::std::convert::Infallible;
    async fn try_recv(mut self) -> Result<Self::Data, Self::Error> {
        Ok(Self::Data {
            email: self.email.recv().await.unwrap(),
            name: self.name.recv().await.unwrap(),
        })
    }
}
struct SendEmailTaskFn;
impl ::adage::TaskFn for SendEmailTaskFn {
    type Input = SendEmailTaskFnInput;
    type Output = ();
    type Error = ::std::convert::Infallible;
    async fn run(input: Self::Input) -> Result<Self::Output, Self::Error> {
        let SendEmailTaskFnInput { email, name } = input;
        let (first, last) = name;
        println!("Sending email to {}", email);
        println!("To: {} {}, Hello!", first, last);
        Ok(())
    }
}
fn send_email(
    email: ::adage::Linker<String>,
    name: ::adage::Linker<(String, String)>,
) -> ::adage::PlannedTask<SendEmailTaskFnInputReceiver, SendEmailTaskFn> {
    let input_receiver = SendEmailTaskFnInputReceiver {
        email: email.link(),
        name: name.link(),
    };
    ::adage::PlannedTask::new(input_receiver)
}

//fn my_flow() {
//    let email = get_email();
//    let name = get_name();
//    send_email(email, name);
//}
fn my_flow(ex: &mut impl ::adage::Executor) {
    let email = get_email().submit(ex);
    let name = get_name().submit(ex);
    send_email(email, name).submit(ex);
}

#[tokio::main]
async fn main() {
    use tracing_subscriber;
    tracing_subscriber::fmt().init();
    tracing::info!("Running examples/simple.rs");

    let mut ex = adage::BasicExecutor::new();
    my_flow(&mut ex);
    use ::adage::Executor;
    ex.run().await;
}
