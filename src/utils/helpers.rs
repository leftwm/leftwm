//! Generic intersection, finding, reordering, and Vec extraction
use std::cmp::Ordering;

pub fn intersect<T>(v1: &[T], v2: &[T]) -> bool
where
    T: PartialEq,
{
    for a in v1 {
        for b in v2 {
            if a == b {
                return true;
            }
        }
    }
    false
}

pub fn vec_extract<T, F>(list: &mut Vec<T>, test: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
    T: Clone,
{
    let len = list.len();
    let mut removed = vec![];
    let mut del = 0;
    {
        let v = &mut **list;

        for i in 0..len {
            if test(&v[i]) {
                removed.push(v[i].clone());
                del += 1;
            } else if del > 0 {
                v.swap(i - del, i);
            }
        }
    }
    list.truncate(len - del);
    removed
}

pub fn cycle_vec<T>(list: &mut Vec<T>, shift: i32)
where
    T: Clone,
{
    let v = &mut **list;
    let change = shift.abs() as usize;
    if v.len() < change {
        return;
    }
    match shift.cmp(&0) {
        Ordering::Less => v.rotate_left(change),
        Ordering::Greater => v.rotate_right(change),
        Ordering::Equal => {}
    }
}

//shifts a object left or right in an Vec by a given amount
pub fn reorder_vec<T, F>(list: &mut Vec<T>, test: F, shift: i32) -> Option<()>
where
    F: Fn(&T) -> bool,
    T: Clone,
{
<<<<<<< HEAD
    let len = list.len() as i32;
    if len < 2 {
<<<<<<< HEAD
        return;
    }
    let (index, item) = match list.iter().enumerate().find(|&x| test(x.1)) {
        Some(x) => (x.0, x.1.clone()),
        None => {
            return;
        }
    };
=======
        return None;
    }
    let index = list.iter().enumerate().find(|&x| test(x.1))?.0;
    let item = list.get(index)?.clone();

>>>>>>> upstream
=======
    if list.len() < 2 {
        return;
    }
    let index = list.iter().enumerate().find(|&x| test(x.1))?.0;
    let item = list.get(index)?.clone();

>>>>>>> refs/remotes/origin/master
    let mut new_index = index as i32 + shift;
    list.remove(index);
    let v = &mut **list;

    if new_index < 0 {
        new_index += len;
        v.rotate_right(1);
    } else if new_index >= len {
        new_index -= len;
        v.rotate_left(1);
    }
    list.insert(new_index as usize, item);
    Some(())
}

pub fn relative_find<T, F>(list: &[T], test: F, shift: i32) -> Option<&T>
where
    F: Fn(&T) -> bool,
    T: Clone,
{
<<<<<<< HEAD
    let len = list.len() as i32;
    if len == 1 {
        return None;
    }
<<<<<<< HEAD
    let index = match list.iter().enumerate().find(|&x| test(x.1)) {
        Some(x) => x.0,
        None => {
            return None;
        }
    };
=======
    let index = list.iter().enumerate().find(|&x| test(x.1))?.0;

>>>>>>> upstream
=======
    if list.len() == 1 {
        return None;
    }
    let index = list.iter().enumerate().find(|&x| test(x.1))?.0;

>>>>>>> refs/remotes/origin/master
    let mut find_index = index as i32 + shift;
    if find_index < 0 {
        find_index += len
    }
    if find_index >= len {
        find_index -= len
    }
    Some(&list[find_index as usize])
}

#[cfg(test)]
pub(crate) mod test {
    pub async fn temp_path() -> std::io::Result<std::path::PathBuf> {
        tokio::task::spawn_blocking(|| tempfile::Builder::new().tempfile_in("target"))
            .await
            .expect("Blocking task joined")?
            .into_temp_path()
            .keep()
            .map_err(Into::into)
    }
}
