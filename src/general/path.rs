use std::fmt::Debug;

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct PathIdent {
    pub ident: String,
    pub template_spec: Vec<Path>,
}

impl PathIdent {
    pub fn is_template(&self) -> bool {
        return !self.template_spec.is_empty();
    }
}

impl Debug for PathIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.template_spec.len() > 0 {
            let mut template = String::new();
            for spec in &self.template_spec {
                template += &format!("{spec:?}, ");
            }
            write!(f, "{}<{template}>", self.ident)
        } else {
            write!(f, "{}", self.ident)
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pub v: Vec<PathIdent>,
}

impl Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.v.iter();
        if let Some(s) = iter.next() {
            write!(f, "{s:?}")?;
        }
        for segment in iter {
            write!(f, "::{segment:?}")?;
        }
        return Ok(());
    }
}

impl<I> From<I> for PathIdent
where
    I: Into<String>,
{
    fn from(value: I) -> Self {
        let ident = value.into();
        return PathIdent {
            ident,
            template_spec: vec![],
        };
    }
}

impl Path {
    pub fn add_segment<I: Into<PathIdent>>(&mut self, s: I) {
        let ident = s.into();
        self.v.push(ident);
    }

    pub fn get(&self, index: usize) -> &PathIdent {
        return &self.v[index];
    }

    pub fn new() -> Self {
        Self { v: vec![] }
    }

    pub fn pop_front(&mut self) {
        self.v.remove(0);
    }

    pub fn len(&self) -> usize {
        return self.v.len();
    }
}

// isn't impl<T> AsRef<T> for T { ... } a default implementation?
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        return self;
    }
}

impl<T> From<T> for Path
where
    T: Into<PathIdent>,
{
    fn from(value: T) -> Self {
        let s = value.into();
        return Self { v: vec![s] };
    }
}
