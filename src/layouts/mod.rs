use super::utils::window::Window;
use super::utils::workspace::Workspace;


pub trait Layout {
    fn update_windows(workspace: &Workspace, windows: Vec<&mut Window>);
}

struct EvenHorizontal{}
impl Layout for EvenHorizontal{
    fn update_windows(workspace: &Workspace, windows: Vec<&mut Window>){
        let width_f = workspace.width as f64 / windows.len() as f64;
        let width = width_f.floor() as i32; 
        let mut x = 0;
        for w in windows {
            w.height = workspace.height;
            w.width = width;
            w.x = workspace.x + x;
            w.y = workspace.y;
            x = x + width;
        }
    }
}
