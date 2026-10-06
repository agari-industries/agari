//! Pins standard scoring. Legal winning hands are built by construction from
//! a fixed seed, scored under default rules, and hashed into one digest. A
//! second run does the same for sanma, with its own seed and digest.
//!
//! A change to either must update its digest in the same commit and say why.
//! To find which hand moved, save the output of
//! `cargo test -p agari --test standard_digest -- --ignored --nocapture`
//! from the old code, then rerun on the new code with
//! `AGARI_DIGEST_BASELINE=<that file>`; set `AGARI_DIGEST_SANMA=1` for the
//! sanma hands.

use agari::context::{GameContext, WinType};
use agari::hand::{HandStructure, KanType, Meld, decompose_hand, decompose_hand_with_melds};
use agari::parse::{CalledMeld, ParsedHand, to_counts, validate_hand_with_context};
use agari::rules::Rules;
use agari::scoring::calculate_score;
use agari::tile::{Honor, Suit, Tile};
use agari::yaku::detect_yaku_with_context;
use sha2::{Digest, Sha256};

const EXPECTED_DIGEST: &str = "c8c5c15b31f1ffaf720ef283e357ed7e44518666bfd61de5f3bfc940e2f78dd8";

const SANMA_DIGEST: &str = "e90ef82ea4f1adb960a5bcb7db0b546a07865ea8f88a2e9c06824567d052c254";

const HANDS: usize = 100_000;
const SEED: u64 = 0x6167_6172_6931;
const SANMA_SEED: u64 = 0x7361_6e6d_6131;

/// SplitMix64. Hand-rolled so the hands never depend on a crate's RNG.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}

const HONORS: [Honor; 7] = [
    Honor::East,
    Honor::South,
    Honor::West,
    Honor::North,
    Honor::White,
    Honor::Green,
    Honor::Red,
];
const WINDS: [Honor; 4] = [Honor::East, Honor::South, Honor::West, Honor::North];
const SANMA_WINDS: [Honor; 3] = [Honor::East, Honor::South, Honor::West];

/// Tiles are indexed 0..34: man, pin, sou (value - 1 within each), honors.
fn tile(index: usize) -> Tile {
    match index {
        0..=8 => Tile::suited(Suit::Man, index as u8 + 1),
        9..=17 => Tile::suited(Suit::Pin, index as u8 - 8),
        18..=26 => Tile::suited(Suit::Sou, index as u8 - 17),
        _ => Tile::honor(HONORS[index - 27]),
    }
}

fn index_of(t: Tile) -> usize {
    match t {
        Tile::Suited { suit, value } => {
            let base = match suit {
                Suit::Man => 0,
                Suit::Pin => 9,
                Suit::Sou => 18,
            };
            base + value as usize - 1
        }
        Tile::Honor(h) => 27 + HONORS.iter().position(|x| *x == h).unwrap(),
    }
}

fn is_suited(index: usize) -> bool {
    index < 27
}

fn value(index: usize) -> usize {
    index % 9 + 1
}

#[derive(Clone, Copy)]
enum Group {
    Seq(usize),
    Trip(usize),
    Kan(usize),
}

/// Tiles a generator may draw from, whether it may form sequences, and
/// whether a sequence may be called (no chi in sanma).
struct Pool {
    tiles: Vec<usize>,
    sequences: bool,
    chi: bool,
}

impl Pool {
    fn new(tiles: Vec<usize>, sequences: bool) -> Self {
        Pool {
            tiles,
            sequences,
            chi: true,
        }
    }

    fn without_chi(mut self) -> Self {
        self.chi = false;
        self
    }

    fn seq_starts(&self) -> Vec<usize> {
        self.tiles
            .iter()
            .copied()
            .filter(|&t| {
                is_suited(t)
                    && value(t) <= 7
                    && self.tiles.contains(&(t + 1))
                    && self.tiles.contains(&(t + 2))
            })
            .collect()
    }

    fn group(&self, rng: &mut Rng) -> Group {
        let starts = self.seq_starts();
        let roll = rng.below(100);
        if self.sequences && !starts.is_empty() && roll < 55 {
            Group::Seq(rng.pick(&starts))
        } else if roll < 92 {
            Group::Trip(rng.pick(&self.tiles))
        } else {
            Group::Kan(rng.pick(&self.tiles))
        }
    }
}

fn all_tiles() -> Vec<usize> {
    (0..34).collect()
}

fn suit_tiles(suit: usize) -> Vec<usize> {
    (suit * 9..suit * 9 + 9).collect()
}

fn honor_tiles() -> Vec<usize> {
    (27..34).collect()
}

fn terminal_tiles() -> Vec<usize> {
    vec![0, 8, 9, 17, 18, 26]
}

/// Sanma leaves 2m-8m out of the wall.
fn sanma_tiles() -> Vec<usize> {
    (0..34).filter(|t| !(1..=7).contains(t)).collect()
}

/// A winning hand before its situation is chosen.
struct Shape {
    concealed: Vec<Tile>,
    called: Vec<Meld>,
}

fn shape_from_groups(rng: &mut Rng, groups: &[Group], pair: usize, chi: bool) -> Option<Shape> {
    let mut counts = [0u8; 34];
    counts[pair] += 2;
    for g in groups {
        match *g {
            Group::Seq(t) => {
                for c in &mut counts[t..t + 3] {
                    *c += 1;
                }
            }
            Group::Trip(t) => counts[t] += 3,
            Group::Kan(t) => counts[t] += 4,
        }
    }
    if counts.iter().any(|&c| c > 4) {
        return None;
    }

    let calls_allowed = rng.chance(60);
    let mut concealed = vec![tile(pair), tile(pair)];
    let mut called = Vec::new();
    for g in groups {
        let call = calls_allowed && rng.chance(35);
        match *g {
            Group::Seq(t) if call && chi => called.push(Meld::shuntsu_open(tile(t))),
            Group::Trip(t) if call => called.push(Meld::koutsu_open(tile(t))),
            Group::Seq(t) => concealed.extend([tile(t), tile(t + 1), tile(t + 2)]),
            Group::Trip(t) => concealed.extend([tile(t); 3]),
            Group::Kan(t) => {
                let kan_type = if !calls_allowed {
                    KanType::Closed
                } else {
                    rng.pick(&[KanType::Closed, KanType::Open, KanType::Added])
                };
                called.push(Meld::kan(tile(t), kan_type));
            }
        }
    }
    Some(Shape { concealed, called })
}

fn random_groups(rng: &mut Rng, pool: &Pool, fixed: &[Group]) -> Vec<Group> {
    let mut groups = fixed.to_vec();
    while groups.len() < 4 {
        groups.push(pool.group(rng));
    }
    groups
}

fn standard_shape(rng: &mut Rng, pool: &Pool, fixed: &[Group], pairs: &[usize]) -> Shape {
    loop {
        let groups = random_groups(rng, pool, fixed);
        let pair = rng.pick(pairs);
        if let Some(shape) = shape_from_groups(rng, &groups, pair, pool.chi) {
            return shape;
        }
    }
}

fn chiitoitsu_shape(rng: &mut Rng, pool: &[usize]) -> Shape {
    let mut chosen: Vec<usize> = Vec::new();
    while chosen.len() < 7 {
        let t = rng.pick(pool);
        if !chosen.contains(&t) {
            chosen.push(t);
        }
    }
    let concealed = chosen.iter().flat_map(|&t| [tile(t), tile(t)]).collect();
    Shape {
        concealed,
        called: Vec::new(),
    }
}

fn kokushi_shape(rng: &mut Rng) -> Shape {
    let orphans: Vec<usize> = terminal_tiles().into_iter().chain(27..34).collect();
    let mut concealed: Vec<Tile> = orphans.iter().map(|&t| tile(t)).collect();
    concealed.push(tile(rng.pick(&orphans)));
    Shape {
        concealed,
        called: Vec::new(),
    }
}

fn chuuren_shape(rng: &mut Rng, suits: &[usize]) -> Shape {
    let base = rng.pick(suits) * 9;
    let mut concealed = Vec::new();
    for v in [0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8] {
        concealed.push(tile(base + v));
    }
    concealed.push(tile(base + rng.below(9)));
    Shape {
        concealed,
        called: Vec::new(),
    }
}

fn tail_shape(rng: &mut Rng) -> Shape {
    let suit = rng.below(3);
    match rng.below(13) {
        0 => standard_shape(
            rng,
            &Pool::new(suit_tiles(suit), true),
            &[],
            &suit_tiles(suit),
        ),
        1 => {
            let tiles: Vec<usize> = suit_tiles(suit).into_iter().chain(27..34).collect();
            standard_shape(rng, &Pool::new(tiles.clone(), true), &[], &tiles)
        }
        2 => {
            if rng.chance(25) {
                chiitoitsu_shape(rng, &honor_tiles())
            } else {
                standard_shape(rng, &Pool::new(honor_tiles(), false), &[], &honor_tiles())
            }
        }
        3 => standard_shape(
            rng,
            &Pool::new(terminal_tiles(), false),
            &[],
            &terminal_tiles(),
        ),
        4 => {
            let tiles: Vec<usize> = terminal_tiles().into_iter().chain(27..34).collect();
            if rng.chance(25) {
                chiitoitsu_shape(rng, &tiles)
            } else {
                standard_shape(rng, &Pool::new(tiles.clone(), false), &[], &tiles)
            }
        }
        5 => standard_shape(rng, &Pool::new(all_tiles(), false), &[], &all_tiles()),
        6 => {
            let green = vec![19, 20, 21, 23, 25, 32];
            standard_shape(rng, &Pool::new(green.clone(), true), &[], &green)
        }
        7 => {
            let dragons = [31, 32, 33];
            let n = 2 + rng.below(2);
            let fixed: Vec<Group> = dragons[..n].iter().map(|&t| Group::Trip(t)).collect();
            let pairs: Vec<usize> = if n == 2 { vec![33] } else { all_tiles() };
            standard_shape(rng, &Pool::new(all_tiles(), true), &fixed, &pairs)
        }
        8 => {
            let n = 3 + rng.below(2);
            let fixed: Vec<Group> = (27..27 + n).map(Group::Trip).collect();
            let pairs: Vec<usize> = if n == 3 { vec![30] } else { all_tiles() };
            standard_shape(rng, &Pool::new(all_tiles(), true), &fixed, &pairs)
        }
        9 => chuuren_shape(rng, &[0, 1, 2]),
        10 => {
            let start = rng.below(7);
            let fixed = [
                Group::Seq(start),
                Group::Seq(9 + start),
                Group::Seq(18 + start),
            ];
            standard_shape(rng, &Pool::new(all_tiles(), true), &fixed, &all_tiles())
        }
        11 => {
            let base = suit * 9;
            let fixed = [Group::Seq(base), Group::Seq(base + 3), Group::Seq(base + 6)];
            standard_shape(rng, &Pool::new(all_tiles(), true), &fixed, &all_tiles())
        }
        _ => {
            let v = rng.below(9);
            let fixed = [Group::Trip(v), Group::Trip(9 + v), Group::Trip(18 + v)];
            standard_shape(rng, &Pool::new(all_tiles(), true), &fixed, &all_tiles())
        }
    }
}

fn shape(rng: &mut Rng) -> Shape {
    let roll = rng.below(100);
    if roll < 70 {
        standard_shape(rng, &Pool::new(all_tiles(), true), &[], &all_tiles())
    } else if roll < 80 {
        chiitoitsu_shape(rng, &all_tiles())
    } else if roll < 83 {
        kokushi_shape(rng)
    } else {
        tail_shape(rng)
    }
}

fn sanma_shape(rng: &mut Rng) -> Shape {
    let tiles = sanma_tiles();
    let roll = rng.below(100);
    if roll < 70 {
        standard_shape(
            rng,
            &Pool::new(tiles.clone(), true).without_chi(),
            &[],
            &tiles,
        )
    } else if roll < 80 {
        chiitoitsu_shape(rng, &tiles)
    } else if roll < 83 {
        kokushi_shape(rng)
    } else if roll < 86 {
        chuuren_shape(rng, &[1, 2])
    } else if roll < 93 {
        let suit = 1 + rng.below(2);
        let one_suit: Vec<usize> = suit_tiles(suit).into_iter().chain(27..34).collect();
        let pool = Pool::new(one_suit.clone(), true).without_chi();
        standard_shape(rng, &pool, &[], &one_suit)
    } else {
        let honors = honor_tiles();
        standard_shape(rng, &Pool::new(honors.clone(), false), &[], &honors)
    }
}

fn meld_tiles(meld: &Meld) -> Vec<Tile> {
    match *meld {
        Meld::Shuntsu(t, _) => {
            let i = index_of(t);
            vec![tile(i), tile(i + 1), tile(i + 2)]
        }
        Meld::Koutsu(t, _) => vec![t; 3],
        Meld::Kan(t, _) => vec![t; 4],
    }
}

/// A shape with its situation, ready to score.
struct Case {
    concealed: Vec<Tile>,
    called: Vec<Meld>,
    context: GameContext,
}

fn random_indicators(rng: &mut Rng, counts: &mut [u8; 34], n: usize, sanma: bool) -> Vec<Tile> {
    let mut indicators = Vec::new();
    while indicators.len() < n {
        let t = rng.below(34);
        if counts[t] < 4 && !(sanma && (1..=7).contains(&t)) {
            counts[t] += 1;
            indicators.push(tile(t));
        }
    }
    indicators
}

fn situate(rng: &mut Rng, shape: Shape, sanma: bool) -> Case {
    let Shape { concealed, called } = shape;
    let mut all: Vec<Tile> = concealed.clone();
    for m in &called {
        all.extend(meld_tiles(m));
    }
    let mut counts = [0u8; 34];
    for t in &all {
        counts[index_of(*t)] += 1;
    }

    let is_open = called.iter().any(|m| m.is_open());
    let has_kan = called.iter().any(|m| matches!(m, Meld::Kan(_, _)));
    let winning_tile = rng.pick(&concealed);
    let win_type = if rng.chance(50) {
        WinType::Tsumo
    } else {
        WinType::Ron
    };
    let winds: &[Honor] = if sanma { &SANMA_WINDS } else { &WINDS };
    let round_wind = rng.pick(winds);
    let seat_wind = rng.pick(winds);

    let mut context =
        GameContext::new(win_type, round_wind, seat_wind).with_winning_tile(winning_tile);
    if is_open {
        context = context.open();
    }

    let first_turn = !is_open && !has_kan && win_type == WinType::Tsumo && rng.chance(3);
    if first_turn {
        context = if seat_wind == Honor::East {
            context.tenhou()
        } else {
            context.chiihou()
        };
    } else if !is_open && rng.chance(40) {
        context = if rng.chance(15) {
            context.double_riichi()
        } else {
            context.riichi()
        };
        if rng.chance(20) {
            context = context.ippatsu();
        }
    }

    let rinshan = win_type == WinType::Tsumo && has_kan && rng.chance(30);
    let chankan = win_type == WinType::Ron && counts[index_of(winning_tile)] == 1 && rng.chance(5);
    if rinshan {
        context = context.rinshan();
    } else if chankan {
        context = context.chankan();
    } else if !first_turn && rng.chance(5) {
        context = context.last_tile();
    }

    let dora_count = 1 + rng.below(5);
    let dora = random_indicators(rng, &mut counts, dora_count, sanma);
    context = context.with_dora(dora);
    if context.is_riichi {
        let ura = random_indicators(rng, &mut counts, dora_count, sanma);
        context = context.with_ura_dora(ura);
    }

    let mut aka = 0;
    for five in [4, 13, 22] {
        if all.contains(&tile(five)) && rng.chance(40) {
            aka += 1;
        }
    }
    context = context.with_aka(aka);

    if sanma {
        let north = index_of(Tile::honor(Honor::North));
        let nukidora = rng.below(5 - counts[north] as usize) as u8;
        context = context.with_rules(Rules::sanma()).with_nukidora(nukidora);
    }

    Case {
        concealed,
        called,
        context,
    }
}

fn tiles_text(tiles: &[Tile]) -> String {
    let mut sorted = tiles.to_vec();
    sorted.sort();
    sorted.iter().map(|t| t.to_string()).collect()
}

/// Only fields that exist today, spelled out, so new context fields cannot
/// move the digest.
fn context_text(c: &GameContext) -> String {
    let list = |v: &[Tile]| v.iter().map(|t| t.to_string()).collect::<String>();
    format!(
        "{:?} wt={} rw={:?} sw={:?} open={} riichi={} double={} ippatsu={} rinshan={} chankan={} last={} tenhou={} chiihou={} dora={} ura={} aka={}",
        c.win_type,
        c.winning_tile.map(|t| t.to_string()).unwrap_or_default(),
        c.round_wind,
        c.seat_wind,
        c.is_open,
        c.is_riichi,
        c.is_double_riichi,
        c.is_ippatsu,
        c.is_rinshan,
        c.is_chankan,
        c.is_last_tile,
        c.is_tenhou,
        c.is_chiihou,
        list(&c.dora_indicators),
        list(&c.ura_dora_indicators),
        c.aka_count,
    )
}

fn score_lines(case: &Case, sanma: bool) -> Vec<String> {
    let mut all = case.concealed.clone();
    for m in &case.called {
        all.extend(meld_tiles(m));
    }
    if sanma {
        let parsed = ParsedHand {
            tiles: case.concealed.clone(),
            aka_count: case.context.aka_count,
            called_melds: case
                .called
                .iter()
                .map(|m| CalledMeld {
                    meld: m.clone(),
                    tiles: meld_tiles(m),
                })
                .collect(),
        };
        validate_hand_with_context(&parsed, &case.context).expect("generated sanma hand is legal");
    }
    let all_counts = to_counts(&all);
    let hand_counts = to_counts(&case.concealed);
    let structures: Vec<HandStructure> = if case.called.is_empty() {
        decompose_hand(&hand_counts)
    } else {
        decompose_hand_with_melds(&hand_counts, &case.called)
    };
    assert!(
        !structures.is_empty(),
        "generated hand does not decompose: {} {:?}",
        tiles_text(&case.concealed),
        case.called
    );

    let mut prefix = format!(
        "{} {:?} | {}",
        tiles_text(&case.concealed),
        case.called,
        context_text(&case.context)
    );
    // Sanma lines carry the pulled Norths; standard lines stay as they were.
    if sanma {
        prefix += &format!(" nukidora={}", case.context.nukidora);
    }
    let mut lines: Vec<String> = structures
        .iter()
        .map(|structure| {
            let y = detect_yaku_with_context(structure, &all_counts, &case.context);
            let s = calculate_score(structure, &y, &case.context);
            let p = &s.payment;
            let nuki = if sanma {
                format!(" nuki={}", y.nuki_dora)
            } else {
                String::new()
            };
            format!(
                "{prefix} | {structure:?} | {:?} han={} dora={}/{}/{}/{}{nuki} yakuman={} | fu={} {:?} han={} {:?} basic={} pay={}/{:?}/{:?}/{:?} dealer={} counted={}",
                y.yaku_list,
                y.total_han,
                y.dora_count,
                y.regular_dora,
                y.ura_dora,
                y.aka_dora,
                y.is_yakuman,
                s.fu.total,
                s.fu.breakdown,
                s.han,
                s.score_level,
                s.basic_points,
                p.total,
                p.from_non_dealer,
                p.from_dealer,
                p.from_discarder,
                s.is_dealer,
                s.is_counted_yakuman,
            )
        })
        .collect();
    lines.sort();
    lines
}

fn all_lines(sanma: bool) -> Vec<Vec<String>> {
    let mut rng = Rng(if sanma { SANMA_SEED } else { SEED });
    (0..HANDS)
        .map(|_| {
            let shape = if sanma {
                sanma_shape(&mut rng)
            } else {
                shape(&mut rng)
            };
            let case = situate(&mut rng, shape, sanma);
            score_lines(&case, sanma)
        })
        .collect()
}

fn digest(sanma: bool) -> String {
    let mut hasher = Sha256::new();
    for lines in all_lines(sanma) {
        for line in lines {
            hasher.update(line.as_bytes());
            hasher.update(b"\n");
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn standard_digest() {
    assert_eq!(
        digest(false),
        EXPECTED_DIGEST,
        "standard scoring changed; see the module doc to find the first differing hand"
    );
}

#[test]
fn sanma_digest() {
    assert_eq!(
        digest(true),
        SANMA_DIGEST,
        "sanma scoring changed; see the module doc to find the first differing hand"
    );
}

#[test]
#[ignore]
fn standard_digest_diff() {
    let hands = all_lines(std::env::var("AGARI_DIGEST_SANMA").as_deref() == Ok("1"));
    let Ok(path) = std::env::var("AGARI_DIGEST_BASELINE") else {
        for (i, lines) in hands.iter().enumerate() {
            for line in lines {
                println!("{i}\t{line}");
            }
        }
        return;
    };
    let baseline = std::fs::read_to_string(&path).expect("read AGARI_DIGEST_BASELINE");
    let current: Vec<String> = hands
        .iter()
        .enumerate()
        .flat_map(|(i, lines)| lines.iter().map(move |line| format!("{i}\t{line}")))
        .collect();
    let old: Vec<&str> = baseline.lines().filter(|l| l.contains('\t')).collect();
    for (n, line) in current.iter().enumerate() {
        match old.get(n) {
            Some(prev) if *prev == line => {}
            prev => {
                println!("first difference at line {n}");
                println!("baseline: {}", prev.unwrap_or(&"<missing>"));
                println!("current:  {line}");
                panic!("scoring differs from the baseline");
            }
        }
    }
    assert_eq!(old.len(), current.len(), "baseline has extra lines");
}
