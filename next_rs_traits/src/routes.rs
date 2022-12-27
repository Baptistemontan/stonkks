use std::{collections::HashMap, any::Any};

pub trait Route {
    fn try_from_parsed_path(segments: &Segments, params: Option<&Params>) -> Option<Box<dyn Any>>;
}

pub struct Params<'a>(HashMap<&'a str, &'a str>);

pub struct Segments<'a>(Box<[&'a str]>);

impl<'a> FromIterator<&'a str> for Segments<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let segments: Box<[&'a str]> = iter.into_iter().collect();
        Segments(segments)
    }
}

impl<'a> Params<'a> {
    pub fn try_parse(params: &'a str) -> Option<Self> {
        let mut params_map = HashMap::new();
        let iter = params.split('&').map(|segment| segment.split_once('='));
        for params in iter {
            let (name, value) = params?;
            params_map.insert(name, value);
        }
        Some(Params(params_map))
    }
}

