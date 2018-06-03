use syn;

// An iterator yielding all conrod attributes in the given attributes.
pub struct ConrodAttrs<I> {
    attrs: I,
}

pub fn conrod_attrs<'a, I>(attrs: I) -> ConrodAttrs<I::IntoIter>
    where I: IntoIterator<Item=&'a syn::Attribute>,
{
    ConrodAttrs { attrs: attrs.into_iter() }
}

impl<'a, I> Iterator for ConrodAttrs<I>
    where I: Iterator<Item=&'a syn::Attribute>,
{
    type Item = Vec<syn::NestedMeta>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(attr) = self.attrs.next() {
            if let Some(_meta) = attr.interpret_meta() {
                if let &syn::Meta::List(ref _metalist) = &_meta{
                    if _metalist.ident == "conrod" {
                        let j = _metalist.nested.clone().into_pairs().map(|pair|pair.into_value()).collect::<Vec<syn::NestedMeta>>();
                        return Some(j);
                    }
                }
            }
        }
        None
    }
}
