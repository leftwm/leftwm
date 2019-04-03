

pub fn intersect<T>(v1: &Vec<T>, v2: &Vec<T> ) -> bool 
    where T: PartialEq
{
    for a in v1{
        for b in v2{
            if a == b {
                return true;
            }
        }
    }
    false
}

pub fn vec_extract<T, F>(list: &mut Vec<T>, test: F ) -> Vec<T>
    where F: Fn(&T) -> bool,
          T: Clone
{
    let mut removed = vec![];
    for x in list.iter() {
        if test(&x) { removed.push(x.clone() ) }
    }
    list.retain(|x| !test(&x) );
    removed
}

pub fn reorder_vec<T, F>(list: &mut Vec<T>, test: F, shift: i32 ) 
    where F: Fn(&T) -> bool,
          T: Clone
{
    let len = list.len() as i32;
    let (index, item) = match list.iter().enumerate().find(|&x| test(&x.1) ){
        Some(x) => (x.0, x.1.clone() ),
        None => { return; }
    };
    list.remove(index);
    let mut new_index = index as i32 + shift;
    if new_index < 0 { new_index += len }
    if new_index >= len { new_index -= len }
    list.insert( new_index as usize, item );
}


pub fn relative_find<T, F>(list: &Vec<T>, test: F, shift: i32 ) -> Option<&T>
    where F: Fn(&T) -> bool,
          T: Clone
{
    let len = list.len() as i32;
    let index = match list.iter().enumerate().find(|&x| test(&x.1) ){
        Some(x) => x.0,
        None => { return None; }
    };
    let mut find_index = index as i32 + shift;
    if find_index < 0 { find_index += len }
    if find_index >= len { find_index -= len }
    Some(&list[find_index as usize ])
}






