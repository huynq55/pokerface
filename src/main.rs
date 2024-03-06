use clap::{Arg, Command};
use rand::seq::SliceRandom;
use rayon::prelude::*;

use std::{cmp::max, collections::HashMap};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Card {
    value: u8,
    suit: u8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum HandRank {
    HighCard(u8, u8, u8, u8, u8),
    OnePair(u8, u8, u8, u8),
    TwoPair(u8, u8, u8),
    ThreeOfAKind(u8, u8, u8),
    Straight(u8),
    Flush(u8, u8, u8, u8, u8),
    FullHouse(u8, u8),
    FourOfAKind(u8, u8),
    StraightFlush(u8),
    RoyalFlush,
}

fn evaluate_hand(hand: &[Card], board: &[Card]) -> HandRank {
    let mut all_cards = hand.to_vec();
    all_cards.extend_from_slice(board);
    all_cards.sort_by(|a, b| a.value.cmp(&b.value));

    if let Some(flush_cards) = check_flush(&all_cards) {
        let straight_values = check_straight(&flush_cards);
        if let Some(straight_values) = straight_values {
            if straight_values.contains(&14) && flush_cards.iter().any(|card| card.value == 10) {
                return HandRank::RoyalFlush;
            } else {
                return HandRank::StraightFlush(*straight_values.iter().max().unwrap());
            }
        }
        return HandRank::Flush(
            flush_cards[0].value,
            flush_cards[1].value,
            flush_cards[2].value,
            flush_cards[3].value,
            flush_cards[4].value,
        );
    }

    let (fours, threes, pairs, singles) = check_multiples(&all_cards);

    if let Some(four_value) = fours {
        if threes.len() == 1 {
            return HandRank::FourOfAKind(four_value, threes[0]);
        } else {
            let potential_kickers = pairs.iter().copied().chain(singles.iter().copied());
            let best_kicker = potential_kickers.max().unwrap_or(0);
            return HandRank::FourOfAKind(four_value, best_kicker);
        }
    } else if threes.len() == 2 || (threes.len() == 1 && pairs.len() >= 1) {
        let full_house_values = if threes.len() == 2 {
            threes
        } else {
            vec![threes[0], pairs[0]]
        };
        return HandRank::FullHouse(full_house_values[0], full_house_values[1]);
    } else if threes.len() == 1 {
        return HandRank::ThreeOfAKind(threes[0], singles[0], singles[1]);
    }

    if let Some(straight_values) = check_straight(&all_cards) {
        return HandRank::Straight(*straight_values.iter().max().unwrap());
    }

    match pairs.len() {
        3 => HandRank::TwoPair(pairs[0], pairs[1], max(pairs[2], singles[0])),
        2 => HandRank::TwoPair(pairs[0], pairs[1], *singles.iter().max().unwrap()),
        1 => HandRank::OnePair(pairs[0], singles[0], singles[1], singles[2]),
        _ => HandRank::HighCard(singles[0], singles[1], singles[2], singles[3], singles[4]),
    }
}

fn compare_hands(hand1: HandRank, hand2: HandRank) -> i32 {
    use HandRank::*;

    match (hand1, hand2) {
        (RoyalFlush, RoyalFlush) => 0,
        (StraightFlush(high_card1), StraightFlush(high_card2)) => {
            high_card1.cmp(&high_card2) as i32
        }
        (FourOfAKind(four1, kicker1), FourOfAKind(four2, kicker2)) => {
            let cards1 = vec![four1, kicker1];
            let cards2 = vec![four2, kicker2];
            compare_cards(&cards1, &cards2)
        }
        (FullHouse(three1, pair1), FullHouse(three2, pair2)) => {
            let cards1 = vec![three1, pair1];
            let cards2 = vec![three2, pair2];
            compare_cards(&cards1, &cards2)
        }
        (Flush(a1, b1, c1, d1, e1), Flush(a2, b2, c2, d2, e2)) => {
            let cards1 = vec![a1, b1, c1, d1, e1];
            let cards2 = vec![a2, b2, c2, d2, e2];
            compare_cards(&cards1, &cards2)
        }
        (Straight(high_card1), Straight(high_card2)) => high_card1.cmp(&high_card2) as i32,
        (
            ThreeOfAKind(three1, kicker1_1, kicker1_2),
            ThreeOfAKind(three2, kicker2_1, kicker2_2),
        ) => {
            let cards1 = vec![three1, kicker1_1, kicker1_2];
            let cards2 = vec![three2, kicker2_1, kicker2_2];
            compare_cards(&cards1, &cards2)
        }
        (TwoPair(high_pair1, low_pair1, kicker1), TwoPair(high_pair2, low_pair2, kicker2)) => {
            let cards1 = vec![high_pair1, low_pair1, kicker1];
            let cards2 = vec![high_pair2, low_pair2, kicker2];
            compare_cards(&cards1, &cards2)
        }
        (
            OnePair(pair1, kicker1_1, kicker1_2, kicker1_3),
            OnePair(pair2, kicker2_1, kicker2_2, kicker2_3),
        ) => {
            let cards1 = vec![pair1, kicker1_1, kicker1_2, kicker1_3];
            let cards2 = vec![pair2, kicker2_1, kicker2_2, kicker2_3];
            compare_cards(&cards1, &cards2)
        }
        (HighCard(a1, b1, c1, d1, e1), HighCard(a2, b2, c2, d2, e2)) => {
            let cards1 = vec![a1, b1, c1, d1, e1];
            let cards2 = vec![a2, b2, c2, d2, e2];
            compare_cards(&cards1, &cards2)
        }
        (_, _) => hand1.partial_cmp(&hand2).unwrap() as i32,
    }
}

fn compare_cards(cards1: &[u8], cards2: &[u8]) -> i32 {
    let mut iter1 = cards1.iter();
    let mut iter2 = cards2.iter();

    loop {
        match (iter1.next(), iter2.next()) {
            (Some(card1), Some(card2)) => {
                let cmp = card1.cmp(card2);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp as i32;
                }
            }
            (None, None) => return 0,
            (Some(_), None) => return 1,
            (None, Some(_)) => return -1,
        }
    }
}

fn simulate_poker_hand(hand: [Card; 2], board: Vec<Card>, num_players: usize) -> (f64, f64, f64) {
    let total_simulations = 1000000;

    let (total_wins, total_ties, total_losses) = (0..total_simulations)
        .into_par_iter()
        .map(|_| {
            let mut deck = create_deck();
            remove_known_cards(&mut deck, &hand, &board);
            deck.shuffle(&mut rand::thread_rng());
            let mut all_hands = vec![hand.clone()];
            let mut simulated_board = board.clone();

            for _ in 0..num_players - 1 {
                all_hands.push([deck.pop().unwrap(), deck.pop().unwrap()]);
            }

            while simulated_board.len() < 5 {
                simulated_board.push(deck.pop().unwrap());
            }

            let player_rank = evaluate_hand(&hand, &simulated_board);

            let mut definitively_loses = false;
            let mut has_tie = false;

            for other_hand in all_hands.iter().skip(1) {
                let other_rank = evaluate_hand(other_hand, &simulated_board);
                let comparison_result = compare_hands(player_rank, other_rank);

                if comparison_result == -1 {
                    definitively_loses = true;
                    break;
                } else if comparison_result == 0 {
                    has_tie = true;
                }
            }

            if definitively_loses {
                (0, 0, 1)
            } else if has_tie {
                (0, 1, 0)
            } else {
                (1, 0, 0)
            }
        })
        .reduce(
            || (0, 0, 0),
            |(wins_a, ties_a, losses_a), (wins_b, ties_b, losses_b)| {
                (wins_a + wins_b, ties_a + ties_b, losses_a + losses_b)
            },
        );

    let win_rate = total_wins as f64 / total_simulations as f64;
    let tie_rate = total_ties as f64 / total_simulations as f64;
    let loss_rate = total_losses as f64 / total_simulations as f64;

    (win_rate, tie_rate, loss_rate)
}

fn remove_known_cards(deck: &mut Vec<Card>, hand: &[Card; 2], board: &Vec<Card>) {
    deck.retain(|card| !hand.contains(card) && !board.contains(card));
}

fn create_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for suit in 0..4 {
        for value in 2..=14 {
            deck.push(Card { value, suit });
        }
    }
    deck
}

fn check_flush(cards: &[Card]) -> Option<Vec<Card>> {
    let mut suits = HashMap::new();

    for card in cards {
        *suits.entry(card.suit).or_insert(0) += 1;
    }

    let flush_suit = suits.into_iter().find(|(_, count)| *count >= 5)?;
    let mut flush_cards: Vec<Card> = cards
        .iter()
        .filter(|card| card.suit == flush_suit.0)
        .cloned()
        .collect();

    if flush_cards.len() >= 5 {
        flush_cards.sort_by(|a, b| b.value.cmp(&a.value));
        Some(flush_cards)
    } else {
        None
    }
}

fn check_straight(cards: &[Card]) -> Option<Vec<u8>> {
    if cards.len() < 5 {
        return None;
    }

    let mut values = cards.iter().map(|card| card.value).collect::<Vec<u8>>();
    values.sort_unstable();
    values.dedup();

    let has_high_ace = values.contains(&14);
    if has_high_ace {
        values.insert(0, 1);
    }

    let mut consecutive_count = 1;
    let mut straight_values = Vec::new();

    for i in 0..values.len() - 1 {
        if values[i] + 1 == values[i + 1] {
            consecutive_count += 1;
            if consecutive_count >= 5 {
                straight_values.push(values[i + 1]);
            }
        } else {
            consecutive_count = 1;
        }
    }

    if straight_values.len() > 0 {
        straight_values.sort_by(|a, b| b.cmp(&a));
        Some(straight_values)
    } else {
        None
    }
}

fn check_multiples(cards: &[Card]) -> (Option<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut counts = HashMap::new();

    for card in cards {
        *counts.entry(card.value).or_insert(0) += 1;
    }

    let mut four = None;
    let mut three = Vec::new();
    let mut pairs = Vec::new();
    let mut singles = Vec::new();

    for (value, count) in counts {
        match count {
            4 => four = Some(value),
            3 => three.push(value),
            2 => pairs.push(value),
            1 => singles.push(value),
            _ => unreachable!(),
        }
    }

    three.sort_unstable_by(|a, b| b.cmp(a));
    pairs.sort_unstable_by(|a, b| b.cmp(a));
    singles.sort_unstable_by(|a, b| b.cmp(a));

    (four, three, pairs, singles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_exists() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![6]));
    }

    #[test]
    fn test_no_straight() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 6, suit: 0 },
            Card { value: 8, suit: 0 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), None);
    }

    #[test]
    fn test_straight_with_duplicates() {
        let cards = vec![
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 0 },
            Card { value: 7, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![7]));
    }

    #[test]
    fn test_straight_with_ace_high() {
        let cards = vec![
            Card { value: 10, suit: 0 },
            Card { value: 11, suit: 0 },
            Card { value: 12, suit: 0 },
            Card { value: 13, suit: 0 },
            Card { value: 14, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![14]));
    }

    #[test]
    fn test_straight_with_ace_low() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 14, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![5]));
    }

    #[test]
    fn test_straight_seven_cards() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 0 },
            Card { value: 7, suit: 0 },
            Card { value: 8, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![8, 7, 6]));
    }

    #[test]
    fn test_straight_seven_non_continuous_cards() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 0 },
            Card { value: 8, suit: 0 },
            Card { value: 9, suit: 0 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![6]));
    }

    #[test]
    fn test_no_pairs_threes_or_fours() {
        let cards = [
            Card { value: 2, suit: 0 },
            Card { value: 4, suit: 1 },
            Card { value: 6, suit: 2 },
            Card { value: 8, suit: 3 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![], vec![], vec![10, 8, 6, 4, 2])
        );
    }

    #[test]
    fn test_one_pair() {
        let cards = [
            Card { value: 3, suit: 0 },
            Card { value: 3, suit: 1 },
            Card { value: 6, suit: 2 },
            Card { value: 8, suit: 3 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![], vec![3], vec![10, 8, 6])
        );
    }

    #[test]
    fn test_two_pairs() {
        let cards = [
            Card { value: 5, suit: 0 },
            Card { value: 5, suit: 1 },
            Card { value: 6, suit: 2 },
            Card { value: 6, suit: 3 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![], vec![6, 5], vec![10])
        );
    }

    #[test]
    fn test_three_pairs() {
        let cards = [
            Card { value: 7, suit: 0 },
            Card { value: 7, suit: 1 },
            Card { value: 5, suit: 2 },
            Card { value: 5, suit: 3 },
            Card { value: 6, suit: 0 },
            Card { value: 6, suit: 1 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![], vec![7, 6, 5], vec![])
        );
    }

    #[test]
    fn test_three_of_a_kind() {
        let cards = [
            Card { value: 7, suit: 0 },
            Card { value: 7, suit: 1 },
            Card { value: 7, suit: 2 },
            Card { value: 8, suit: 3 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![7], vec![], vec![10, 8])
        );
    }

    #[test]
    fn test_double_three_of_a_kind() {
        let cards = [
            Card { value: 7, suit: 0 },
            Card { value: 8, suit: 1 },
            Card { value: 7, suit: 2 },
            Card { value: 8, suit: 0 },
            Card { value: 7, suit: 1 },
            Card { value: 8, suit: 2 },
        ];
        assert_eq!(check_multiples(&cards), (None, vec![8, 7], vec![], vec![]));
    }

    #[test]
    fn test_four_of_a_kind() {
        let cards = [
            Card { value: 9, suit: 0 },
            Card { value: 9, suit: 1 },
            Card { value: 9, suit: 2 },
            Card { value: 9, suit: 3 },
            Card { value: 10, suit: 0 },
        ];
        assert_eq!(check_multiples(&cards), (Some(9), vec![], vec![], vec![10]));
    }

    #[test]
    fn test_a_pair_and_three_of_a_kind() {
        let cards = [
            Card { value: 2, suit: 0 },
            Card { value: 2, suit: 1 },
            Card { value: 2, suit: 2 },
            Card { value: 3, suit: 3 },
            Card { value: 3, suit: 0 },
        ];
        assert_eq!(check_multiples(&cards), (None, vec![2], vec![3], vec![]));
    }

    #[test]
    fn test_full_house_1() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 3, suit: 3 },
            Card { value: 3, suit: 0 },
            Card { value: 3, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FullHouse(3, 2));
    }
    #[test]
    fn test_full_house_2() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 3, suit: 0 },
            Card { value: 3, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FullHouse(2, 3));
    }
    #[test]
    fn test_straight_flush() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 3, suit: 0 }];
        let boards = [
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 1 },
            Card { value: 8, suit: 0 },
        ];
        assert_ne!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(6));
    }

    #[test]
    fn test_straight_flush_2() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 3, suit: 0 }];
        let boards = [
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 6, suit: 0 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(6));
    }

    #[test]
    fn test_compare_hands_three_1() {
        let hand1 = HandRank::ThreeOfAKind(10, 9, 8);
        let hand2 = HandRank::ThreeOfAKind(10, 9, 8);
        assert_eq!(compare_hands(hand1, hand2), 0);
    }

    #[test]
    fn test_compare_hands_three_2() {
        let hand1 = HandRank::ThreeOfAKind(10, 9, 6);
        let hand2 = HandRank::ThreeOfAKind(10, 8, 6);
        assert_eq!(compare_hands(hand1, hand2), 1);
    }

    #[test]
    fn test_compare_hands_three_3() {
        let hand1 = HandRank::ThreeOfAKind(10, 9, 8);
        let hand2 = HandRank::ThreeOfAKind(10, 9, 7);
        assert_eq!(compare_hands(hand1, hand2), 1);
    }

    #[test]
    fn test_compare_hands_flush_1() {
        let hand1 = HandRank::Flush(10, 9, 8, 7, 4);
        let hand2 = HandRank::Flush(10, 9, 8, 7, 5);
        assert_eq!(compare_hands(hand1, hand2), -1);
    }

    #[test]
    fn test_compare_hands_two_pairs_1() {
        let hand1 = HandRank::TwoPair(10, 9, 8);
        let hand2 = HandRank::TwoPair(10, 9, 7);
        assert_eq!(compare_hands(hand1, hand2), 1);
    }

    #[test]
    fn test_compare_hands_one_pairs_1() {
        let hand1 = HandRank::OnePair(10, 9, 8, 7);
        let hand2 = HandRank::OnePair(10, 9, 8, 6);
        assert_eq!(compare_hands(hand1, hand2), 1);
    }

    #[test]
    fn test_compare_hands_tie_1() {
        let hand1 = HandRank::Flush(10, 9, 8, 7, 4);
        let hand2 = HandRank::Flush(10, 9, 8, 7, 4);
        assert_eq!(compare_hands(hand1, hand2), 0);
    }

    #[test]
    fn test_compare_hands_tie_2() {
        let hand1 = HandRank::HighCard(10, 9, 8, 7, 4);
        let hand2 = HandRank::HighCard(10, 9, 8, 7, 4);
        assert_eq!(compare_hands(hand1, hand2), 0);
    }

    #[test]
    fn test_four_of_a_kind_1() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 2, suit: 3 },
            Card { value: 9, suit: 0 },
            Card { value: 8, suit: 0 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FourOfAKind(2, 9));
    }

    #[test]
    fn test_four_of_a_kind_2() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 2, suit: 3 },
            Card { value: 6, suit: 0 },
            Card { value: 8, suit: 0 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FourOfAKind(2, 8));
    }

    #[test]
    fn test_four_of_a_kind_3() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 2, suit: 3 },
            Card { value: 7, suit: 0 },
            Card { value: 7, suit: 1 },
            Card { value: 7, suit: 2 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FourOfAKind(2, 7));
    }

    #[test]
    fn test_four_of_a_kind_4() {
        let cards = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
        let boards = [
            Card { value: 2, suit: 2 },
            Card { value: 2, suit: 3 },
            Card { value: 6, suit: 0 },
            Card { value: 10, suit: 1 },
            Card { value: 5, suit: 2 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FourOfAKind(2, 10));
    }

    #[test]
    fn test_straight_ace_both_high_and_low() {
        let cards = vec![
            Card { value: 10, suit: 0 },
            Card { value: 11, suit: 1 },
            Card { value: 12, suit: 2 },
            Card { value: 13, suit: 3 },
            Card { value: 14, suit: 0 },
            Card { value: 2, suit: 1 },
        ];
        assert_eq!(check_straight(&cards), Some(vec![14]));
    }

    #[test]
    fn test_two_three_of_a_kinds() {
        let cards = [
            Card { value: 6, suit: 0 },
            Card { value: 6, suit: 1 },
            Card { value: 6, suit: 2 },
            Card { value: 8, suit: 3 },
            Card { value: 8, suit: 0 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(check_multiples(&cards), (None, vec![8, 6], vec![], vec![]));
    }

    #[test]
    fn test_straight_and_one_pair() {
        let cards = [
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 1 },
            Card { value: 4, suit: 2 },
            Card { value: 5, suit: 3 },
            Card { value: 6, suit: 0 },
            Card { value: 6, suit: 2 },
        ];
        assert_eq!(
            check_multiples(&cards),
            (None, vec![], vec![6], vec![5, 4, 3, 2])
        );
    }

    #[test]
    fn test_full_house_tiebreaker() {
        let cards1 = [Card { value: 2, suit: 0 }, Card { value: 5, suit: 1 }];
        let cards2 = [Card { value: 3, suit: 0 }, Card { value: 4, suit: 0 }];

        let boards = [
            Card { value: 2, suit: 1 },
            Card { value: 2, suit: 2 },
            Card { value: 3, suit: 1 },
            Card { value: 3, suit: 2 },
        ];

        assert_eq!(evaluate_hand(&cards1, &boards), HandRank::FullHouse(2, 3));
        assert_eq!(evaluate_hand(&cards2, &boards), HandRank::FullHouse(3, 2));

        assert_eq!(
            compare_hands(
                evaluate_hand(&cards2, &boards),
                evaluate_hand(&cards1, &boards)
            ),
            1
        );
    }

    #[test]
    fn test_straight_flush_ace_low() {
        let cards = [Card { value: 14, suit: 0 }, Card { value: 3, suit: 0 }];
        let boards = [
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 2, suit: 0 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(5));
    }

    #[test]
    fn test_straight_ace_low() {
        let cards = [Card { value: 14, suit: 0 }, Card { value: 3, suit: 0 }];
        let boards = [
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 2, suit: 1 },
            Card { value: 8, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::Straight(5));
    }

    #[test]
    fn test_straight_flush_1() {
        let cards = [Card { value: 8, suit: 0 }, Card { value: 8, suit: 1 }];
        let boards = [
            Card { value: 9, suit: 0 },
            Card { value: 10, suit: 0 },
            Card { value: 11, suit: 0 },
            Card { value: 12, suit: 0 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(12));
    }

    #[test]
    fn test_full_house_3() {
        let cards = [Card { value: 5, suit: 0 }, Card { value: 5, suit: 1 }];
        let boards = [
            Card { value: 5, suit: 2 },
            Card { value: 9, suit: 3 },
            Card { value: 9, suit: 0 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FullHouse(5, 9));
    }

    #[test]
    fn test_full_house_tiebreaker_2() {
        let cards = [Card { value: 7, suit: 0 }, Card { value: 7, suit: 1 }];

        let boards = [
            Card { value: 7, suit: 1 },
            Card { value: 5, suit: 2 },
            Card { value: 5, suit: 3 },
            Card { value: 8, suit: 0 },
            Card { value: 4, suit: 0 },
        ];

        assert_eq!(evaluate_hand(&cards, &boards), HandRank::FullHouse(7, 5));
    }

    #[test]
    fn test_high_card_multiple_kickers() {
        let cards = [Card { value: 3, suit: 0 }, Card { value: 5, suit: 1 }];
        let boards = [
            Card { value: 6, suit: 3 },
            Card { value: 7, suit: 0 },
            Card { value: 9, suit: 0 },
            Card { value: 11, suit: 2 },
            Card { value: 13, suit: 1 },
        ];
        assert_eq!(
            evaluate_hand(&cards, &boards),
            HandRank::HighCard(13, 11, 9, 7, 6)
        );
    }

    #[test]
    fn test_flush_1() {
        let cards = [Card { value: 5, suit: 1 }, Card { value: 8, suit: 1 }];
        let boards = [
            Card { value: 8, suit: 2 },
            Card { value: 14, suit: 0 },
            Card { value: 4, suit: 1 },
            Card { value: 14, suit: 1 },
            Card { value: 6, suit: 1 },
        ];
        assert_eq!(
            evaluate_hand(&cards, &boards),
            HandRank::Flush(14, 8, 6, 5, 4)
        );
    }

    #[test]
    fn test_straight_flush_3() {
        let cards = [Card { value: 8, suit: 1 }, Card { value: 8, suit: 2 }];
        let boards = [
            Card { value: 10, suit: 1 },
            Card { value: 11, suit: 1 },
            Card { value: 12, suit: 1 },
            Card { value: 9, suit: 1 },
            Card { value: 13, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(13));
    }

    #[test]
    fn test_royal_flush() {
        let cards = [Card { value: 9, suit: 1 }, Card { value: 9, suit: 2 }];
        let boards = [
            Card { value: 10, suit: 1 },
            Card { value: 11, suit: 1 },
            Card { value: 12, suit: 1 },
            Card { value: 13, suit: 1 },
            Card { value: 14, suit: 0 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::StraightFlush(13));
    }

    #[test]
    fn test_royal_flush_2() {
        let cards = [Card { value: 9, suit: 1 }, Card { value: 9, suit: 2 }];
        let boards = [
            Card { value: 10, suit: 1 },
            Card { value: 11, suit: 1 },
            Card { value: 12, suit: 1 },
            Card { value: 13, suit: 1 },
            Card { value: 14, suit: 1 },
        ];
        assert_eq!(evaluate_hand(&cards, &boards), HandRank::RoyalFlush);
    }
}

fn parse_cards(input: &str) -> Vec<Card> {
    input
        .split_whitespace()
        .filter_map(|card_str| {
            if card_str.len() != 2 {
                None
            } else {
                let bytes = card_str.as_bytes();
                let value = match bytes[0] as char {
                    '2'..='9' => bytes[0] as u8 - b'0',
                    'T' => 10,
                    'J' => 11,
                    'Q' => 12,
                    'K' => 13,
                    'A' => 14,
                    _ => panic!("Invalid card value"),
                };
                let suit = match bytes[1] as char {
                    'h' => 0,
                    'd' => 1,
                    'c' => 2,
                    's' => 3,
                    _ => panic!("Invalid card suit"),
                };
                Some(Card { value, suit })
            }
        })
        .collect()
}

fn main() {
    let matches = Command::new("Poker Hand Simulator")
        .version("1.0")
        .author("Your Name")
        .about("Simulates a poker hand")
        .arg(
            Arg::new("hand")
                .short('h')
                .long("hand")
                .value_name("HAND")
                .help("Sets the hand to evaluate")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("board")
                .short('b')
                .long("board")
                .value_name("BOARD")
                .help("Sets the board cards")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    let hand_input = matches.value_of("hand").unwrap();
    let board_input = matches.value_of("board").unwrap();

    let hand_vec = parse_cards(hand_input);
    let board_vec = parse_cards(board_input);

    if hand_vec.len() != 2 {
        panic!(
            "Invalid hand length: expected 2 cards, found {}",
            hand_vec.len()
        );
    }

    let hand_array = [hand_vec[0], hand_vec[1]];

    for num_players in 2..=5 {
        let (win_rate, tie_rate, _) =
            simulate_poker_hand(hand_array, board_vec.clone(), num_players);

        println!(
            "Number of players: {}. Simulated Win rate: {:.2}%, Simulated Tie rate: {:.2}%, EV 1$ bet {:.2}$",
            num_players,
            win_rate * 100.0,
            tie_rate * 100.0,
            num_players as f64 * win_rate + tie_rate - 1.0
        );
    }
}
