//! Structures defining the sockets on items.


/// A color of an item or socket.
///
/// In PoE, this is associated with a particular main stat.
///
/// *Note*: Although it does appear as such in the API,
/// "abyss" is not a color so it's not included here.
/// See `ItemSockets::abyss_count` for the number of abyss sockets an item has.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum Color {
    /// Red gem or socket, associated with Strength.
    #[serde(rename = "R")]
    Red,
    /// Green gem or socket, associated with Dexterity.
    #[serde(rename = "G")]
    Green,
    /// Blue gem or socket, associated with Intelligence.
    #[serde(rename = "B")]
    Blue,
    /// White gem or socket (not associated with any stat)
    #[serde(rename = "W")]
    White,
}


/// Sockets an item has, if any.
#[derive(Debug)]
pub struct ItemSockets {
    /// Number of abyss sockets the item has.
    pub(crate) abyssal_count: u64,
    /// Groups of regular sockets that are linked together.
    pub(crate) regular_groups: Vec<SocketGroup>,
}

impl Default for ItemSockets {
    fn default() -> Self {
        // Default is no sockets at all, for items that don't have them at all.
        ItemSockets {
            abyssal_count: 0,
            regular_groups: vec![],
        }
    }
}

impl ItemSockets {
    /// Number of regular sockets this item has.
    pub fn regular_count(&self) -> u64 {
        self.regular_groups.iter().map(|g| g.size() as u64).sum()
    }

    /// Number of abyssal sockets this item has.
    #[inline]
    pub fn abyssal_count(&self) -> u64 {
        self.abyssal_count
    }

    /// Colors of all regular sockets (in an unspecified order).
    #[inline]
    pub fn colors<'s>(&'s self) -> Box<Iterator<Item=Color> + 's> {
        Box::new(self.regular_groups.iter().flat_map(|g| g.colors.iter().cloned()))
    }

    /// Linked groups of regular sockets.
    pub fn links<'s>(&'s self) -> Box<Iterator<Item=Box<Iterator<Item=Color> + 's>> + 's> {
        Box::new(
            self.regular_groups.iter().map(|g| {
                Box::new(g.colors.iter().cloned()) as Box<Iterator<Item=Color>>
            })
        )
    }

    /// Maximum number of linked sockets on the item.
    ///
    /// If an item is said to be N-linked (e.g. 5-linked), this will be N.
    #[inline]
    pub fn max_links(&self) -> usize {
        self.regular_groups.iter().map(|g| g.size()).max().unwrap_or(0)
    }
}

/// A group of linked sockets on an item.
#[derive(Debug)]
pub struct SocketGroup {
    /// ID of the socket group, assigned by the API.
    ///
    /// This is a small integer index starting from 0.
    pub(crate) id: u8,
    /// Colors of linked sockets.
    pub(crate) colors: Vec<Color>,
}

impl SocketGroup {
    /// Size of the socket group (number of sockets therein).
    #[inline]
    pub fn size(&self) -> usize {
        self.colors.len()
    }

    /// How many sockets of a particular color are there in the group.
    #[inline]
    pub fn count_of(&self, color: Color) -> usize {
        self.colors.iter().filter(|&&c| c == color).count()
    }
}
