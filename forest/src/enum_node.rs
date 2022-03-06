/// Create an `enum` from a list of types implementing [crate::tree::Node] with members made from a list of types which implement [crate::tree::Node].
/// TODO:
/// 1. use Chunk types as parameters. Access view types via Chunk::View.
/// 2. move logic from [indirect_nav] into here.
macro_rules! fromMembers {
    ( $(($name:ident, $child:ty)),+ $(,)?) => {
        #[derive(::derive_more::From)]
        pub enum Child<'a> {$(
            $name($child),
        )*}

        #[derive(::derive_more::From)]
        pub enum TraitView<'a> {$(
            $name(<$name<'a> as crate::tree::NodeNav<$child>>::TTraitChildren),
        )*}

        #[derive(Clone)]
        pub enum NodeView<'a> {$(
            $name($name<'a>),
        )*}

        impl<'a> crate::tree::NodeNav<Child<'a>> for NodeView<'a> {
            type TTraitChildren = TraitView<'a>;
            type TLabels = LabelIterator<'a>;

            fn get_traits(&self) -> Self::TLabels {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeNav::<$child>::get_traits(n).into(),
                )*}
            }

            fn get_trait(&self, label: crate::tree::Label) -> Self::TTraitChildren {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeNav::<$child>::get_trait(n, label).into(),
                )*}
            }
        }

        impl<'a> crate::tree::NodeData for NodeView<'a> {
            fn get_def(&self) -> crate::tree::Def {
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeData::get_def(n),
                )*}
            }
            fn get_payload(&self) -> Option<crate::util::ImSlice>{
                match self {$(
                    NodeView::$name(n) => crate::tree::NodeData::get_payload(n),
                )*}
            }
        }

        impl<'a> crate::node_id::HasId for NodeView<'a> {
            fn get_id(&self) -> crate::node_id::NodeId {
                match self {$(
                    NodeView::$name(n) => crate::node_id::HasId::get_id(n),
                )*}
            }
        }

        impl<'a> Iterator for TraitView<'a> {
            type Item = Child<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                match self {$(
                    TraitView::$name(ref mut c) => c.next().map(|c| c.into()),
                )*}
            }
        }

        #[derive(::derive_more::From)]
        pub enum LabelIterator<'a> {$(
            $name(<$name<'a> as crate::tree::NodeNav<$child>>::TLabels),
        )*}

        impl Iterator for LabelIterator<'_> {
            type Item = crate::tree::Label;

            fn next(&mut self) -> Option<Self::Item> {
                match self {$(
                    LabelIterator::$name(ref mut c) => c.next(),
                )*}
            }
        }
    }
}

pub(crate) use fromMembers;
