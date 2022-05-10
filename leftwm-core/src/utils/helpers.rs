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

pub fn cycle_vec<T>(list: &mut Vec<T>, shift: i32) -> Option<()>
where
    T: Clone,
{
    let v = &mut **list;
    let change = shift.unsigned_abs() as usize;
    if v.len() < change {
        return None;
    }
    match shift.cmp(&0) {
        Ordering::Less => v.rotate_left(change),
        Ordering::Greater => v.rotate_right(change),
        Ordering::Equal => {}
    }
    Some(())
}

//shifts a object left or right in an Vec by a given amount
pub fn reorder_vec<T, F>(list: &mut Vec<T>, test: F, shift: i32) -> Option<()>
where
    F: Fn(&T) -> bool,
    T: Clone,
{
    let len = list.len() as i32;
    if len < 2 {
        return None;
    }
    let index = list.iter().position(test)?;
    let item = list.get(index)?.clone();

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

/// Find element relative to reference element.
///
/// eg. to get the next element, use `shift` 1,
/// to get the previous element, use `shift` -1.
///
/// ## Arguments
/// * `list` - The list to get the element from
/// * `reference_finder` - Predicate to find the reference element in the list
/// * `shift` - The shift (distance) of the element you try to find relative to the reference element, can be negative to move left
/// * `should_loop` - If the list should loop when the `shift` goes beyond the start/end of the list
///
/// ## Example
/// ```
/// let list = vec!["hello", "world", "foo", "bar"];
/// let result = leftwm_core::utils::helpers::relative_find(&list, |&e| e == "world", 2, false);
/// assert_eq!(result, Some(&"bar"));
/// ```
pub fn relative_find<T, F>(
    list: &[T],
    reference_finder: F,
    shift: i32,
    should_loop: bool,
) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    let len = list.len() as i32;
    let reference_index = list.iter().position(reference_finder)?;
    let loops = if shift.is_negative() {
        // check if shift is larger than there are elements on the left
        shift.unsigned_abs() as usize > reference_index
    } else {
        // check if shift is larger than there are elements on the right
        shift as usize > len as usize - (reference_index + 1)
    };

    let relative_index = if loops && !should_loop {
        None
    } else {
        let shift = shift % len;
        let shifted_index = reference_index as i32 + shift;
        let max_index = len - 1;
        if shifted_index < 0 {
            Some((len + shifted_index) as usize)
        } else if shifted_index > max_index {
            Some((shifted_index - len) as usize)
        } else {
            Some(shifted_index as usize)
        }
    }?;

    list.get(relative_index)
}

#[cfg(test)]
pub(crate) mod test {
    use crate::utils::helpers::relative_find;

    pub async fn temp_path() -> std::io::Result<std::path::PathBuf> {
        tokio::task::spawn_blocking(|| tempfile::Builder::new().tempfile_in("../target"))
            .await
            .expect("Blocking task joined")?
            .into_temp_path()
            .keep()
            .map_err(Into::into)
    }

    #[test]
    fn relative_find_should_work_both_ways() {
        let list = vec!["hello", "world", "foo", "bar"];
        let result = relative_find(&list, |&e| e == "hello", 2, false);
        assert_eq!(result, Some(&"foo"));
        let result = relative_find(&list, |&e| e == "bar", -2, false);
        assert_eq!(result, Some(&"world"));
    }

    #[test]
    fn relative_find_with_inexistent_reference_must_return_none() {
        let list = vec!["hello", "world", "foo", "bar"];
        let result = relative_find(&list, |&e| e == "inexistent", 2, false);
        assert_eq!(result, None);
    }

    #[test]
    fn relative_find_should_be_able_to_loop() {
        let list = vec!["hello", "world", "foo", "bar"];
        let result = relative_find(&list, |&e| e == "hello", 4, true);
        assert_eq!(result, Some(&"hello"));
        let result = relative_find(&list, |&e| e == "hello", 9, true);
        assert_eq!(result, Some(&"world"));
        let result = relative_find(&list, |&e| e == "hello", -9, true);
        assert_eq!(result, Some(&"bar"));
    }

    #[test]
    fn relative_find_loop_can_be_disabled() {
        let list = vec!["hello", "world", "foo", "bar"];
        let result = relative_find(&list, |&e| e == "hello", 9, false);
        assert_eq!(result, None);
    }
}
