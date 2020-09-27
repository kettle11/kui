use kui::*;

fn main() {
    run_async(ui);
}

async fn ui(app: Application, events: Events) {
    let mut ui = SimpleUI::new(app, events);
    let inter_medium = ui.new_font(include_bytes!("../resources/Inter-Medium.ttf"));

    let mut counter: u32 = 0;

    loop {
        ui.update().await;

        let body = ui.edit().font(inter_medium);
        let row = body.spaced_row(10.);

        if button(&row, "Increment counter") {
            counter += 1;
        }

        row.text(&format!("{:?}", counter));
    }
}
