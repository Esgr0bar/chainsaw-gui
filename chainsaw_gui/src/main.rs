mod chainsaw_app;
mod utils;

use chainsaw_app::ChainsawApp;

fn main() {
    let mut app = ChainsawApp::default();
    app.run();
}
