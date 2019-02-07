//use super::display_servers::DisplayServer;
//use super::manager::Manager;

//pub fn load_config<T: DisplayServer>(manager: &mut Manager<T>) {
//    // default to tags 1 to 9
//    for i in 1..10 {
//        manager.tags.push(i.to_string());
//    }
//}
//
//#[test]
//fn default_config_should_create_tags_1_to_9() {
//    use super::display_servers::MockDisplayServer;
//    let mut subject: Manager<MockDisplayServer> = Manager::new();
//    load_config(&mut subject);
//    let tags = subject.tags.clone();
//    assert!(tags[0] == "1", "default tag {1} did not load");
//    assert!(tags[1] == "2", "default tag {2} did not load");
//    assert!(tags[2] == "3", "default tag {3} did not load");
//    assert!(tags[3] == "4", "default tag {4} did not load");
//    assert!(tags[4] == "5", "default tag {5} did not load");
//    assert!(tags[5] == "6", "default tag {6} did not load");
//    assert!(tags[6] == "7", "default tag {6} did not load");
//    assert!(tags[7] == "8", "default tag {7} did not load");
//    assert!(tags[8] == "9", "default tag {8} did not load");
//}
