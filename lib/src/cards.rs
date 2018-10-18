use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Green,
    Yellow,
    Red,
    Pirate,
}

impl Step {
    pub fn next(self) -> Step {
        match self {
            Step::Green => Step::Yellow,
            Step::Yellow => Step::Red,
            Step::Red => Step::Pirate,
            _ => panic!("invalid next step"),
        }
    }
    pub fn prev(self) -> Step {
        match self {
            Step::Green | Step::Yellow => Step::Green,
            Step::Red => Step::Yellow,
            Step::Pirate => Step::Pirate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Capacity {
    None,
    Life(i8),
    Card(i8),
    Destroy,
    Double,
    Copy,
    LowerStep,
    Sort,
    Swap(i8),
    UnderDeck,
    EndLife(i8),
    MaxZero,
    Stop,
}

impl fmt::Display for Capacity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Capacity::*;

        let s = match *self {
            None => "...",
            Life(1) => "+1 life",
            Life(2) => "+2 life",
            Card(1) => "+1 card",
            Card(2) => "+2 cards",
            Destroy => "1× destroy",
            Double => "1× double",
            Copy => "1× copy",
            LowerStep => "-1 step",
            Sort => "Sort 3 cards",
            Swap(1) => "1× exchange",
            Swap(2) => "2× exchange",
            UnderDeck => "1× under the pile",
            EndLife(1) => "-1 life",
            EndLife(2) => "-2 life",
            MaxZero => "Highest card = 0",
            Stop => "Stop",
            c => panic!("Unknown capacity {:?}", c),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct FightingCard {
    pub title: String,
    pub fighting_value: isize,
    pub capacity: Capacity,
}

impl fmt::Display for FightingCard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \"{}\"", self.fighting_value, self.capacity)
    }
}

impl FightingCard {
    fn new(title: &str, fighting_value: isize, capacity: Capacity) -> FightingCard {
        FightingCard {
            title: title.to_string(),
            fighting_value,
            capacity,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Hazard {
    Leveled([u8; 3]),
    Pirate(u8),
    PirateTwiceLife(u8),
    PirateAging,
    PirateHalf(u8),
    PirateHazard,
    PirateAdd(u8),
}

#[derive(Debug, Clone)]
pub struct HazardCard {
    pub title: String,
    pub free_cards: u8,
    pub hazard: Hazard,
}

impl HazardCard {
    fn new(title: &str, free_cards: u8, hazard: Hazard) -> HazardCard {
        HazardCard {
            title: title.to_string(),
            free_cards,
            hazard,
        }
    }
}

impl fmt::Display for HazardCard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "free:{} levels:{:?}", self.free_cards, self.hazard)
    }
}

#[derive(Debug, Clone)]
pub enum CardKind {
    Starting(FightingCard),
    AgingNormal(FightingCard),
    AgingDifficult(FightingCard),
    HazardKnowledge(HazardCard, FightingCard),
    Pirate(HazardCard),
}

impl fmt::Display for CardKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str.push_str(&self.to_fighting_card().to_string());
        let aging = match self {
            CardKind::AgingNormal(_) => " (aging 1)",
            CardKind::AgingDifficult(_) => " (aging 2)",
            _ => "",
        };
        str.push_str(aging);
        write!(f, "{}", str)
    }
}

impl CardKind {
    pub fn to_hazard_card(&self) -> &HazardCard {
        match self {
            CardKind::HazardKnowledge(card, _) => card,
            CardKind::Pirate(card) => card,
            _ => panic!("to_hazard_card()"),
        }
    }

    pub fn to_fighting_card(&self) -> &FightingCard {
        match self {
            CardKind::Starting(card) => card,
            CardKind::AgingNormal(card) => card,
            CardKind::AgingDifficult(card) => card,
            CardKind::HazardKnowledge(_, card) => card,
            _ => panic!("to_fighting_card()"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CardDescription {
    pub kind: CardKind,
    pub start_qty: usize,
    pub filename: String,
}

impl fmt::Display for CardDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl CardDescription {
    fn new(kind: CardKind, start_qty: usize, filename: &str) -> Self {
        Self {
            kind,
            start_qty,
            filename: filename.to_string(),
        }
    }

    fn new_starting(
        title: &str,
        fighting_value: isize,
        start_qty: usize,
        filename: &str,
    ) -> CardDescription {
        CardDescription {
            kind: CardKind::Starting(FightingCard::new(title, fighting_value, Capacity::None)),
            start_qty,
            filename: filename.to_string(),
        }
    }

    fn new_aging_normal(
        title: &str,
        fighting_value: isize,
        start_qty: usize,
        filename: &str,
    ) -> CardDescription {
        CardDescription {
            kind: CardKind::AgingNormal(FightingCard::new(title, fighting_value, Capacity::None)),
            start_qty,
            filename: filename.to_string(),
        }
    }

    fn new_aging_difficult(
        title: &str,
        fighting_value: isize,
        start_qty: usize,
        filename: &str,
    ) -> CardDescription {
        CardDescription {
            kind: CardKind::AgingDifficult(FightingCard::new(
                title,
                fighting_value,
                Capacity::None,
            )),
            start_qty,
            filename: filename.to_string(),
        }
    }

    fn new_hazard_knowledge(
        hazard: HazardCard,
        knowledge: FightingCard,
        start_qty: usize,
        filename: &str,
    ) -> CardDescription {
        CardDescription {
            kind: CardKind::HazardKnowledge(hazard, knowledge),
            start_qty,
            filename: filename.to_string(),
        }
    }

    fn new_pirate(title: &str, free_cards: u8, hazard: Hazard, filename: &str) -> CardDescription {
        CardDescription {
            kind: CardKind::Pirate(HazardCard::new(title, free_cards, hazard)),
            start_qty: 1,
            filename: filename.to_string(),
        }
    }

    pub fn is_starting(&self) -> bool {
        // FIXME is there a simpler way for is_starting/is_pirate ? ...
        match self.kind {
            CardKind::Starting(..) => true,
            _ => false,
        }
    }

    pub fn is_pirate(&self) -> bool {
        match self.kind {
            CardKind::Pirate(..) => true,
            _ => false,
        }
    }

    pub fn is_aging_normal(&self) -> bool {
        match self.kind {
            CardKind::AgingNormal(..) => true,
            _ => false,
        }
    }

    pub fn is_aging_difficult(&self) -> bool {
        match self.kind {
            CardKind::AgingDifficult(..) => true,
            _ => false,
        }
    }

    pub fn is_hazard_knowledge(&self) -> bool {
        match self.kind {
            CardKind::HazardKnowledge(..) => true,
            _ => false,
        }
    }

    pub fn is_very_stupid(&self) -> bool {
        match self.kind {
            CardKind::AgingNormal(FightingCard {
                fighting_value: -3, ..
            }) => true,
            _ => false,
        }
    }

    pub fn get_destroy_value(&self) -> Option<isize> {
        match self.kind {
            CardKind::Starting(..) | CardKind::HazardKnowledge(..) => Some(1),
            CardKind::AgingNormal(..) | CardKind::AgingDifficult(..) => Some(2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card<'a> {
    pub description: &'a CardDescription,
    pub tapped: bool,
    pub destroy: bool,
    pub double: bool,
}

impl<'a> Card<'a> {
    pub fn new(description: &'a CardDescription) -> Self {
        Self {
            description,
            tapped: false,
            destroy: false,
            double: false,
        }
    }

    pub fn get_fighting_value(&self) -> isize {
        if self.destroy {
            return 0;
        }
        let mut val = self.description.kind.to_fighting_card().fighting_value as isize;
        if self.double {
            val *= 2;
        }
        val
    }

    pub fn reset(&mut self) {
        self.tapped = false;
        self.double = false;
    }
}

impl<'a> fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        str.push_str(&self.description.to_string());
        if self.tapped {
            str.push_str(" tapped");
        }
        if self.double {
            str.push_str(" double");
        }
        if self.destroy {
            str.push_str(" destroy");
        }
        write!(f, "{}", str)
    }
}

fn all_card_description() -> [CardDescription; 48] {
    use self::CardKind::*;

    [
        CardDescription::new(
            Starting(FightingCard::new("eating", 0, Capacity::Life(2))),
            1,
            "friday-030.jpg",
        ),
        CardDescription::new_starting("genius", 2, 1, "friday-033.jpg"),
        CardDescription::new_starting("focused", 1, 3, "friday-031.jpg"),
        CardDescription::new_starting("weak", 0, 8, "friday-038.jpg"),
        CardDescription::new_starting("distracted", -1, 5, "friday-043.jpg"),
        CardDescription::new_aging_normal("stupid", -2, 2, "friday-058.jpg"),
        CardDescription::new(
            AgingNormal(FightingCard::new("very tired", 0, Capacity::Stop)),
            1,
            "friday-057.jpg",
        ),
        CardDescription::new_aging_normal("very stupid", -3, 1, "friday-056.jpg"),
        CardDescription::new_aging_normal("distracted", -1, 1, "friday-055.jpg"),
        CardDescription::new(
            AgingNormal(FightingCard::new("afraid", 0, Capacity::MaxZero)),
            2,
            "friday-053.jpg",
        ),
        CardDescription::new(
            AgingNormal(FightingCard::new("hungry", 0, Capacity::EndLife(-1))),
            1,
            "friday-051.jpg",
        ),
        CardDescription::new(
            AgingNormal(FightingCard::new("very hungry", 0, Capacity::EndLife(-2))),
            1,
            "friday-048.jpg",
        ),
        CardDescription::new_aging_difficult("self homicidal", -5, 1, "friday-049.jpg"),
        CardDescription::new_aging_difficult("idiot", -4, 1, "friday-050.jpg"),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("reader", 0, Capacity::LowerStep),
            1,
            "friday-009.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("trick", 0, Capacity::UnderDeck),
            1,
            "friday-022.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("knowledge", 0, Capacity::Destroy),
            1,
            "friday-011.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("mimicry", 0, Capacity::Copy),
            1,
            "friday-016.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("nutriment", 0, Capacity::Life(1)),
            2,
            "friday-027.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("equipment", 0, Capacity::Card(2)),
            2,
            "friday-000.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wreck boat", 1, Hazard::Leveled([0, 1, 3])),
            FightingCard::new("strategy", 0, Capacity::Swap(2)),
            2,
            "friday-001.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("mimicry", 1, Capacity::Copy),
            1,
            "friday-015.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("mimicry", 1, Capacity::Destroy),
            1,
            "friday-023.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("repetition", 1, Capacity::Double),
            1,
            "friday-017.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("repetition", 1, Capacity::UnderDeck),
            1,
            "friday-029.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("nutriment", 1, Capacity::Life(1)),
            2,
            "friday-025.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring the island", 2, Hazard::Leveled([1, 3, 6])),
            FightingCard::new("weapon", 2, Capacity::None),
            2,
            "friday-014.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("experience", 2, Capacity::Card(1)),
            1,
            "friday-012.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("knowledge", 2, Capacity::Destroy),
            1,
            "friday-003.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("vision", 2, Capacity::Sort),
            1,
            "friday-002.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("strategy", 2, Capacity::Swap(2)),
            1,
            "friday-013.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("nutriment", 2, Capacity::Life(1)),
            1,
            "friday-005.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Exploring deep the island", 3, Hazard::Leveled([2, 5, 8])),
            FightingCard::new("repetition", 2, Capacity::Double),
            1,
            "friday-008.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wild animals", 4, Hazard::Leveled([4, 7, 11])),
            FightingCard::new("knowledge", 3, Capacity::Destroy),
            1,
            "friday-018.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wild animals", 4, Hazard::Leveled([4, 7, 11])),
            FightingCard::new("experience", 3, Capacity::Card(1)),
            1,
            "friday-019.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wild animals", 4, Hazard::Leveled([4, 7, 11])),
            FightingCard::new("vision", 3, Capacity::Sort),
            1,
            "friday-004.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Wild animals", 4, Hazard::Leveled([4, 7, 11])),
            FightingCard::new("strategy", 3, Capacity::Swap(2)),
            1,
            "friday-024.jpg",
        ),
        CardDescription::new_hazard_knowledge(
            HazardCard::new("Cannibal", 5, Hazard::Leveled([5, 9, 14])),
            FightingCard::new("weapon", 4, Capacity::None),
            2,
            "friday-006.jpg",
        ),
        CardDescription::new_pirate("", 10, Hazard::Pirate(40), "friday-061.jpg"),
        CardDescription::new_pirate(
            "Each additonal fighting card costs 2 life points",
            7,
            Hazard::PirateTwiceLife(16),
            "friday-062.jpg",
        ),
        CardDescription::new_pirate("", 8, Hazard::Pirate(30), "friday-063.jpg"),
        CardDescription::new_pirate("", 6, Hazard::Pirate(20), "friday-064.jpg"),
        CardDescription::new_pirate(
            "+2 hazard points for each aging card added to your Robinson stack",
            5,
            Hazard::PirateAging,
            "friday-065.jpg",
        ),
        CardDescription::new_pirate(
            "Only half of the fighting cards count (aging cards must be part of this)",
            9,
            Hazard::PirateHalf(22),
            "friday-066.jpg",
        ),
        CardDescription::new_pirate(
            "Fight against all remaining hazard cards",
            0,
            Hazard::PirateHazard,
            "friday-067.jpg",
        ),
        CardDescription::new_pirate("", 7, Hazard::Pirate(25), "friday-068.jpg"),
        CardDescription::new_pirate("", 9, Hazard::Pirate(35), "friday-069.jpg"),
        CardDescription::new_pirate(
            "Each drawn fighting card counts +1 fighting point",
            10,
            Hazard::PirateAdd(52),
            "friday-070.jpg",
        ),
    ]
}

lazy_static! {
    pub static ref CARDS: [CardDescription; 48] = all_card_description();
}
