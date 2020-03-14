use std::convert::From;

use super::progress::Progress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
}

impl<S: ToString, T> From<&(S, Progress<T>)> for Item {
    fn from(pair: &(S, Progress<T>)) -> Self {
        let (id, pg) = pair;
        let pg = pg.lock();
        Item {
            id: id.to_string(),
            name: pg.name.clone(),
            total: pg.total,
            size: pg.size,
            canceled: pg.canceled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_test() {
        let id = "id".to_string();
        let name = "name".to_string();
        let pg = Progress::new(&name, ());
        let item: Item = (&(id.clone(), pg)).into();
        let ans = Item {
            id: id.to_string(),
            name,
            total: 0,
            size: 0,
            canceled: false,
        };
        assert_eq!(item, ans);
    }

    #[test]
    fn writed_progress_test() {
    }
}
