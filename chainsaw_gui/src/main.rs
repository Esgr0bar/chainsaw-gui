mod chainsaw_app;
mod utils;

use chainsaw_app::ChainsawApp;

fn main() {
    let _app = ChainsawApp::default();
    ChainsawApp::run();
}
