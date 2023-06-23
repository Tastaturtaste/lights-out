slint::include_modules!();

use slint::Model;
fn main() -> Result<(), slint::PlatformError> {
    let ui = LightsOut::new()?;
    ui.on_lit_from_activated(|activations| {
        dbg!(activations.row_count());
        let active: Vec<bool> = dbg!(activations.iter().collect());
        std::rc::Rc::new(slint::VecModel::from(light_buttons(&active))).into()
    });
    ui.run()
}

fn light_buttons(activations: &[bool]) -> Vec<bool>{
    let mut lights = activations.to_vec();
    lights.push(false);
    lights
}