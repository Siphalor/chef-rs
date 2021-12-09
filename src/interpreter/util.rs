use std::collections::BTreeMap;

pub struct LazyTreeMap<'a, K: Ord + Clone, V: Clone> {
    value_factory: fn(&K)->V,
    parent: LazyTreeMapParent<'a, K, V>,
    pub delegate: BTreeMap<K, V>,
}

pub enum LazyTreeMapParent<'a, K: Ord + Clone, V: Clone> {
    None,
    Map(&'a BTreeMap<K, V>),
    Other(&'a LazyTreeMap<'a, K, V>)
}

impl<'a, K: Ord + Clone, V: Clone> LazyTreeMap<'a, K, V> {
    pub fn new(parent: LazyTreeMapParent<'a, K, V>, constructor: fn(&K)->V) -> LazyTreeMap<'a, K, V> {
        LazyTreeMap{
            parent,
            value_factory: constructor,
            delegate: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.delegate.get(key).or_else(|| {
            match self.parent {
                LazyTreeMapParent::None => { None }
                LazyTreeMapParent::Map(map) => {
                    map.get(&key)
                },
                LazyTreeMapParent::Other(other) => {
                    other.get(&key)
                }
            }
        })
    }

    pub fn get_mut(&mut self, key: K) -> &mut V {
        if self.delegate.contains_key(&key) {
            return self.delegate.get_mut(&key).unwrap();
        }

        let mut value: Option<V> = None;
        match self.parent {
            LazyTreeMapParent::Map(map) => {
                value = map.get(&key).map(V::clone);
            },
            LazyTreeMapParent::Other(other) => {
                value = other.get(&key).clone().map(V::clone);
            },
            LazyTreeMapParent::None => {}
        }
        let value = value.unwrap_or_else(|| (self.value_factory)(&key));
        self.delegate.insert(key.clone(), value);
        return self.delegate.get_mut(&key).unwrap();
    }
}

pub fn read_number(buffer: &mut String) -> f64 {
    loop {
        if buffer.is_empty() {
            std::io::stdin().read_line(buffer).expect("failed to read from STDIN");
            continue;
        }
        let mut chars = buffer.chars().peekable();
        let mut val = 0f64;
        let mut neg = false;
        let mut int_part = false;
        let mut dec_part = false;
        let mut dec_precision = 0.1f64;

        loop {
            let sym = chars.peek();
            if let Some(sym) = sym {
                if dec_part {
                    if sym.is_numeric() {
                        val += sym.to_digit(10).unwrap() as f64 * dec_precision;
                        dec_precision /= 10.0;
                        chars.next();
                    } else {
                        break;
                    }
                } else if int_part {
                    if *sym == '.' {
                        dec_part = true;
                        chars.next();
                    } else if sym.is_numeric() {
                        val = val * 10.0 + sym.to_digit(10).unwrap() as f64;
                        chars.next();
                    } else {
                        break;
                    }
                } else {
                    if *sym == '-' {
                        neg = true;
                    } else if sym.is_numeric() {
                        val += sym.to_digit(10).unwrap() as f64;
                        int_part = true;
                    } else {
                        neg = false;
                    }
                    chars.next();
                }
            } else {
                break;
            }
        }

        *buffer = chars.collect();

        if int_part {
            if neg {
                val *= -1.0;
            }
            return val;
        }
    }
}

pub fn read_char(buffer: &mut String) -> u32 {
    while buffer.is_empty() {
        std::io::stdin().read_line(buffer).expect("failed to read from STDIN");
    }
    let mut chars = buffer.chars();
    let res = chars.next().unwrap() as u32;
    *buffer = chars.collect();
    return res;
}
