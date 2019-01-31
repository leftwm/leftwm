
use x11_dl::xlib;


type MockHandle = i32;
#[derive(Debug, Clone, PartialEq)]
pub enum Handle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window)
}




#[derive(Debug, Clone)]
pub struct Window {
    pub handle: Handle,
    pub name: Option<String>,
    pub tags: Vec<String>,
}


impl Window{

    pub fn new(h: Handle, name: Option<String>) -> Window{
        Window{
            handle:h,
            name:name,
            tags: Vec::new()
        }
    }


    pub fn tag( &mut self, tag: String ){
        if tag == "" { return }
        for t in &self.tags {
            if t == &tag { return }
        }
        self.tags.push( tag );
    }

    pub fn has_tag( &self, tag: String ) -> bool {
        for t in &self.tags {
            if t == &tag { return true }
        }
        false
    }

    pub fn untag( &mut self, tag: String ) {
        let mut new_tags: Vec<String> = Vec::new();
        for t in &self.tags {
            if t != &tag { new_tags.push( t.clone() ) }
        }
        self.tags = new_tags;
    }


}





#[test]
fn should_be_able_to_tag_a_window(){
    let mut subject = Window::new( Handle::MockHandle(1), None);
    subject.tag("test".to_string() );
    assert!( subject.has_tag("test".to_string() ) , "was unable to tag the window");
}

#[test]
fn should_be_able_to_untag_a_window(){
    let mut subject = Window::new( Handle::MockHandle(1), None);
    subject.tag("test".to_string() );
    subject.untag("test".to_string() );
    assert!( subject.has_tag("test".to_string() ) == false , "was unable to untag the window");
}



