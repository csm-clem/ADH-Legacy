//! Filtres de visibilité des pastilles.
//!
//! Remplace `NoteMasking` et `PartielsMasking` (23 `Q_PROPERTY` booléens avec
//! 23 signaux) par deux tableaux de booléens : l'UI en mode immédiat relit l'état
//! à chaque image, il n'y a plus rien à notifier.

use crate::note::{Note, NOTE_COUNT};
use crate::partial::{Partial, PARTIAL_COUNT};

macro_rules! mask {
    ($name:ident, $item:ty, $all:expr, $count:expr) => {
        #[doc = concat!("Visibilité de chaque ", stringify!($item), ".")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            visible: [bool; $count],
        }

        impl Default for $name {
            fn default() -> Self {
                Self { visible: $all }
            }
        }

        impl $name {
            /// Tous visibles (comportement d'ouverture de l'Atlas).
            pub fn all_visible() -> Self {
                Self::default()
            }

            /// Tous masqués.
            pub fn none_visible() -> Self {
                Self {
                    visible: [false; $count],
                }
            }

            /// Visibilité actuelle de cet élément.
            pub fn is_visible(&self, item: $item) -> bool {
                self.visible[item.index()]
            }

            /// Force la visibilité de cet élément.
            pub fn set(&mut self, item: $item, visible: bool) {
                self.visible[item.index()] = visible;
            }

            /// Rend cet élément visible.
            pub fn show(&mut self, item: $item) {
                self.visible[item.index()] = true;
            }

            /// Masque cet élément.
            pub fn hide(&mut self, item: $item) {
                self.visible[item.index()] = false;
            }

            /// Bascule la visibilité et renvoie le nouvel état.
            pub fn toggle(&mut self, item: $item) -> bool {
                let index = item.index();
                self.visible[index] = !self.visible[index];
                self.visible[index]
            }

            /// Rend tous les éléments visibles.
            pub fn show_all(&mut self) {
                self.visible = [true; $count];
            }

            /// Masque tous les éléments.
            pub fn hide_all(&mut self) {
                self.visible = [false; $count];
            }

            /// Nombre d'éléments actuellement visibles.
            pub fn visible_count(&self) -> usize {
                self.visible.iter().filter(|v| **v).count()
            }

            /// Indexes des éléments visibles.
            pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
                (0..$count).filter(move |i| self.visible[*i])
            }
        }
    };
}

mask!(NoteMask, Note, [true; NOTE_COUNT], NOTE_COUNT);
mask!(PartialMask, Partial, [true; PARTIAL_COUNT], PARTIAL_COUNT);

impl NoteMask {
    /// Vrai si la pastille de cette note doit être dessinée.
    pub fn shows(&self, note: Note) -> bool {
        self.is_visible(note)
    }
}

impl PartialMask {
    /// Vrai si les harmoniques de ce partiel doivent être dessinées.
    pub fn shows(&self, partial: Partial) -> bool {
        self.is_visible(partial)
    }
}
