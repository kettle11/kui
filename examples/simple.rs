use kui::*;

fn main() {
	run("Hello", ui);
}

async fn ui(context: UIContext) {	
    let mut counter: u32 = 0;
	loop {
        let mut ui = context.next().await;
        let body = ui.edit();

        if button(&body, id!(), "Increment counter") {
            counter += 1;
            println!("COUNTER: {:?}", counter);
        }
	}
}