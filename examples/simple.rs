use kui::*;

fn main() {
	run("Hello", ui);
}

async fn ui(context: UIContext) {	
    let inter_medium;
    {
        let mut ui = context.next().await;
        inter_medium = ui.font_from_bytes(include_bytes!("../resources/Inter-Medium.ttf"));
    }
    
    let mut counter: u32 = 0;
	loop {
        let mut ui = context.next().await;
        let body = ui.edit().font(inter_medium);

        if button(&body, id!(), "Increment counter") {
            counter += 1;
            println!("COUNTER: {:?}", counter);
        }
	}
}