use kui::*;

fn main() {
    run_async(ui);
}

async fn ui(app: Application, events: Events) {
    let mut ui = SimpleUI::new(app, events);
    let inter_medium = ui
        .ui
        .font_from_bytes(include_bytes!("../resources/Inter-Medium.ttf"));

    let mut counter: u32 = 0;

    loop {
        ui.update().await;
        
        let body = ui.edit().font(inter_medium);

        if button(&body, id!(), "Increment counter") {
            counter += 1;
            println!("COUNTER: {:?}", counter);
        }
    }
}
