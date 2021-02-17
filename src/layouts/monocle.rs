use crate::models::Window;
use crate::models::Workspace;
// use crate::models::WindowState;

/// Layout which gives only one window with the full desktop realestate. A monocle mode.
pub fn update(workspace: &Workspace, windows: &mut Vec<&mut &mut Window>) {
    let window_count = windows.len();

    if window_count == 0 {
        return;
    }

    let mut iter = windows.iter_mut(); 

       //maximize primary window
        
    {
        if let Some(monowin) = iter.next() {
            // let mut monoclestate: Vec<WindowState> = Vec::new();

            //Above, MaximizedVert, MaximizedHorz
            // monoclestate.push(WindowState::Above);
            // monoclestate.push(WindowState::MaximizedVert);
            // monoclestate.push(WindowState::MaximizedHorz);
            monowin.margin = 0;
            monowin.set_height(workspace.height());
            monowin.set_width(workspace.width());
            monowin.set_x(workspace.x());
            monowin.set_y(workspace.y());

            // monowin.set_states(monoclestate.to_vec());
            monowin.set_visible(true);
        }
    }
        
        //hide all other windows
     {
        if window_count > 1 {
            for w in iter {
                w.set_visible(false); //window state Hide
            }
        }
    }
}
