use std::cmp::max;

use clap::{Arg, Command};
use rand::{seq::SliceRandom, thread_rng}; // Sử dụng crate rand để xáo bài
use rayon::prelude::*;

// Định nghĩa cấu trúc cho một lá bài và bàn tay
#[derive(Clone, Copy, Debug, PartialEq)]
struct Card {
    value: u8, // Giá trị của lá bài, 2 đến 14, với 11 là J, 12 là Q, v.v.
    suit: u8,  // Chất của lá bài, có thể định nghĩa từ 0 đến 3
}

// Định nghĩa các loại bàn tay
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum HandRank {
    HighCard(u8, u8, u8, u8, u8),
    OnePair(u8, u8, u8, u8),
    TwoPair(u8, u8, u8),
    ThreeOfAKind(u8),
    Straight(u8),
    Flush(u8, u8, u8, u8, u8),
    FullHouse(u8, u8),
    FourOfAKind(u8, u8),
    StraightFlush(u8),
    RoyalFlush,
}

impl Card {
    fn display(&self) -> String {
        let value_str = match self.value {
            2..=10 => self.value.to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            14 => "A".to_string(),
            _ => panic!("Giá trị bài không hợp lệ"),
        };

        let suit_str = match self.suit {
            0 => "♥", // Cơ
            1 => "♦", // Rô
            2 => "♣", // Tép
            3 => "♠", // Bích
            _ => panic!("Chất bài không hợp lệ"),
        };

        format!("{}{}", value_str, suit_str)
    }
}

// Một số hàm cơ bản có thể cần thiết

// fn evaluate_hand(hand: &[Card], board: &[Card]) -> HandRank { ... }
// Hàm đánh giá bàn tay
fn evaluate_hand(hand: &[Card], board: &[Card]) -> HandRank {
    // Kết hợp các lá bài trên tay và trên bàn
    let mut all_cards = hand.to_vec();
    all_cards.extend_from_slice(board);

    // Sắp xếp các lá bài theo giá trị
    all_cards.sort_by(|a, b| a.value.cmp(&b.value));

    // Kiểm tra các loại bàn tay khác nhau
    let flush_cards = check_flush(&all_cards);
    let straight_high_card = check_straight(&all_cards);
    let (four, three, pairs, singles) = check_multiples(&all_cards);

    // Đánh giá loại bàn tay dựa trên kết quả kiểm tra
    match (
        flush_cards,
        straight_high_card,
        four,
        three,
        pairs.len(),
        singles,
    ) {
        (Some(_flush_cards), Some(14), _, _, _, _) => HandRank::RoyalFlush,
        (Some(_flush_cards), _, _, _, _, _)
            if {
                if let Some(highest_card) = check_straight(&_flush_cards) {
                    true
                } else {
                    false
                }
            } =>
        {
            let highest_card = check_straight(&_flush_cards).unwrap();
            HandRank::StraightFlush(highest_card)
        }
        (_, _, Some(_four), Some(_three), _, _) => HandRank::FourOfAKind(_four, _three),
        (_, _, Some(_four), _, 1, singles) => {
            if pairs[0] > singles[0] {
                HandRank::FourOfAKind(_four, pairs[0])
            } else {
                HandRank::FourOfAKind(_four, singles[0])
            }
        }
        (_, _, Some(_four), _, _, singles) => {
            HandRank::FourOfAKind(_four, max(max(singles[0], singles[1]), singles[2]))
        }
        (_, _, _, Some(three_card), _, _) => {
            // Xác định liệu có thêm một bộ ba khác không, không giống bộ ba hiện tại
            let other_cards = all_cards
                .iter()
                .filter(|c| c.value != three_card)
                .collect::<Vec<&Card>>();
            let has_another_three = other_cards.iter().any(|&card| {
                other_cards
                    .iter()
                    .filter(|&&c| c.value == card.value)
                    .count()
                    == 3
            });

            if has_another_three {
                // Tìm giá trị của bộ ba thứ hai
                let other_three_value = other_cards
                    .iter()
                    .find_map(|&card| {
                        if other_cards
                            .iter()
                            .filter(|&&c| c.value == card.value)
                            .count()
                            == 3
                        {
                            Some(card.value)
                        } else {
                            None
                        }
                    })
                    .unwrap(); // Có thể sử dụng unwrap vì chúng ta biết chắc chắn có một giá trị

                // Xác định xem đâu là bộ ba lớn hơn và đâu là đôi
                let (higher, lower) = if three_card > other_three_value {
                    (three_card, other_three_value)
                } else {
                    (other_three_value, three_card)
                };

                HandRank::FullHouse(higher, lower)
            } else if pairs.len() > 0 {
                // Nếu không có bộ ba thứ hai nhưng có một đôi
                HandRank::FullHouse(three_card, pairs[0])
            } else {
                // Không có đôi hoặc bộ ba thứ hai
                HandRank::ThreeOfAKind(three_card)
            }
        }
        (Some(_flush_cards), _, _, _, _, _) => {
            // Tạo Flush object từ singles
            HandRank::Flush(
                _flush_cards[0].value,
                _flush_cards[1].value,
                _flush_cards[2].value,
                _flush_cards[3].value,
                _flush_cards[4].value,
            )
        }

        (_, Some(high_card), _, _, _, _) => HandRank::Straight(high_card),
        (_, _, _, _, 2, singles) => HandRank::TwoPair(pairs[0], pairs[1], singles[0]),
        (_, _, _, _, 1, singles) => HandRank::OnePair(pairs[0], singles[0], singles[1], singles[2]),
        (_, _, _, _, _, singles) => {
            HandRank::HighCard(singles[0], singles[1], singles[2], singles[3], singles[4])
        }
    }
}

// fn compare_hands(hand1: HandRank, hand2: HandRank) -> i32 { ... }
// Hàm so sánh hai bàn tay
fn compare_hands(hand1: HandRank, hand2: HandRank) -> i32 {
    if hand1 > hand2 {
        1 // hand1 thắng
    } else if hand1 < hand2 {
        -1 // hand2 thắng
    } else {
        // Trường hợp cả hai bàn tay có cùng xếp hạng
        match hand1 {
            HandRank::RoyalFlush => 0, // Hòa khi cả hai đều là Royal Flush
            HandRank::StraightFlush(high_card1) => {
                if let HandRank::StraightFlush(high_card2) = hand2 {
                    high_card1.cmp(&high_card2) as i32
                } else {
                    0 // Không thể xảy ra, chỉ để đảm bảo mã không lỗi
                }
            }
            HandRank::FourOfAKind(card1, kicker1) => {
                if let HandRank::FourOfAKind(card2, kicker2) = hand2 {
                    match card1.cmp(&card2) {
                        std::cmp::Ordering::Equal => kicker1.cmp(&kicker2) as i32,
                        other => other as i32,
                    }
                } else {
                    0
                }
            }
            HandRank::FullHouse(three1, pair1) => {
                if let HandRank::FullHouse(three2, pair2) = hand2 {
                    match three1.cmp(&three2) {
                        std::cmp::Ordering::Equal => pair1.cmp(&pair2) as i32,
                        other => other as i32,
                    }
                } else {
                    0
                }
            }
            HandRank::Flush(a1, b1, c1, d1, e1) => {
                if let HandRank::Flush(a2, b2, c2, d2, e2) = hand2 {
                    if a1 != a2 {
                        return a1.cmp(&a2) as i32;
                    } else if b1 != b2 {
                        return b1.cmp(&b2) as i32;
                    } else if c1 != c2 {
                        return c1.cmp(&c2) as i32;
                    } else if d1 != d2 {
                        return d1.cmp(&d2) as i32;
                    } else {
                        return e1.cmp(&e2) as i32;
                    }
                } else {
                    0
                }
            }
            HandRank::Straight(high_card1) => {
                if let HandRank::Straight(high_card2) = hand2 {
                    high_card1.cmp(&high_card2) as i32
                } else {
                    0
                }
            }
            HandRank::ThreeOfAKind(card1) => {
                if let HandRank::ThreeOfAKind(card2) = hand2 {
                    card1.cmp(&card2) as i32
                } else {
                    0
                }
            }
            HandRank::TwoPair(high_pair1, low_pair1, kicker1) => {
                if let HandRank::TwoPair(high_pair2, low_pair2, kicker2) = hand2 {
                    match high_pair1.cmp(&high_pair2) {
                        std::cmp::Ordering::Equal => match low_pair1.cmp(&low_pair2) {
                            std::cmp::Ordering::Equal => kicker1.cmp(&kicker2) as i32,
                            other => other as i32,
                        },
                        other => other as i32,
                    }
                } else {
                    0
                }
            }
            HandRank::OnePair(pair1, kicker1, kicker2, kicker3) => {
                if let HandRank::OnePair(pair2, kicker2_1, kicker2_2, kicker2_3) = hand2 {
                    // 1. So sánh giá trị pair:
                    match pair1.cmp(&pair2) {
                        std::cmp::Ordering::Greater => return 1,
                        std::cmp::Ordering::Less => return -1,
                        std::cmp::Ordering::Equal => {
                            // 2. Nếu pair bằng nhau, so sánh kicker1:
                            match kicker1.cmp(&kicker2_1) {
                                std::cmp::Ordering::Greater => return 1,
                                std::cmp::Ordering::Less => return -1,
                                std::cmp::Ordering::Equal => {
                                    // 3. Nếu kicker1 bằng nhau, so sánh kicker2:
                                    match kicker2.cmp(&kicker2_2) {
                                        std::cmp::Ordering::Greater => return 1,
                                        std::cmp::Ordering::Less => return -1,
                                        std::cmp::Ordering::Equal => {
                                            // 4. Nếu kicker2 bằng nhau, so sánh kicker3:
                                            match kicker3.cmp(&kicker2_3) {
                                                std::cmp::Ordering::Greater => return 1,
                                                std::cmp::Ordering::Less => return -1,
                                                std::cmp::Ordering::Equal => return 0, // Hòa
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    0
                }
            }
            HandRank::HighCard(a1, b1, c1, d1, e1) => {
                if let HandRank::Flush(a2, b2, c2, d2, e2) = hand2 {
                    if a1 != a2 {
                        return a1.cmp(&a2) as i32;
                    } else if b1 != b2 {
                        return b1.cmp(&b2) as i32;
                    } else if c1 != c2 {
                        return c1.cmp(&c2) as i32;
                    } else if d1 != d2 {
                        return d1.cmp(&d2) as i32;
                    } else {
                        return e1.cmp(&e2) as i32;
                    }
                } else {
                    0
                }
            }
        }
    }
}

// Hàm chính để mô phỏng và tính toán xác suất
fn simulate_poker_hand(hand: [Card; 2], board: Vec<Card>, num_players: usize) -> (f64, f64) {
    let total_simulations = 1000000; // Số lần mô phỏng

    let (total_wins, total_losses) = (0..total_simulations)
        .into_par_iter() // Biến đổi sang Parallel Iterator
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
            let mut has_worse_hand = false;

            for other_hand in all_hands.iter().skip(1) {
                let other_rank = evaluate_hand(other_hand, &simulated_board);
                if compare_hands(player_rank, other_rank) != 1 {
                    has_worse_hand = true;
                    break;
                }
            }

            if has_worse_hand {
                (0, 1)
            } else {
                (1, 0)
            }
        })
        .reduce(
            || (0, 0), // Giá trị khởi tạo cho mỗi phân đoạn
            |(wins_a, losses_a), (wins_b, losses_b)| {
                // Kết hợp hai phần kết quả
                (wins_a + wins_b, losses_a + losses_b)
            },
        );

    // Tính tỷ lệ thắng thua
    let win_rate = total_wins as f64 / total_simulations as f64;
    let loss_rate = total_losses as f64 / total_simulations as f64;

    (win_rate, loss_rate)
}

// Hàm này sẽ loại bỏ các lá bài đã biết khỏi bộ bài
fn remove_known_cards(deck: &mut Vec<Card>, hand: &[Card; 2], board: &Vec<Card>) {
    deck.retain(|card| !hand.contains(card) && !board.contains(card));
}

// Tạo một bộ bài mới
fn create_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for suit in 0..4 {
        for value in 2..=14 {
            deck.push(Card { value, suit });
        }
    }
    deck
}

// Hàm kiểm tra Flush
fn check_flush(cards: &[Card]) -> Option<Vec<Card>> {
    // Tạo một mảng để đếm số lượng lá bài cho mỗi chất
    let mut suits = [0; 4]; // Một mảng với 4 phần tử, tương ứng với 4 chất

    // Đếm số lá bài cho mỗi chất
    for card in cards {
        suits[card.suit as usize] += 1;
    }

    // Tìm kiếm chất có ít nhất 5 lá bài
    let flush_suit = suits.iter().position(|&count| count >= 5)?;

    // Lấy 5 lá bài đầu tiên của chất đó
    let mut flush_cards = Vec::with_capacity(5);
    for card in cards {
        if card.suit as usize == flush_suit {
            flush_cards.push(card.clone());
            if flush_cards.len() == 5 {
                break;
            }
        }
    }

    // Trả về 5 lá bài Flush nếu tìm thấy, hoặc None nếu không
    if flush_cards.len() == 5 {
        flush_cards.sort_by(|a, b| b.value.cmp(&a.value));
        Some(flush_cards)
    } else {
        None
    }
}

// Hàm kiểm tra Straight
fn check_straight(cards: &[Card]) -> Option<u8> {
    if cards.len() < 5 {
        return None; // Cần ít nhất 5 lá bài để tạo thành một Straight
    }

    let mut values = cards.iter().map(|card| card.value).collect::<Vec<u8>>();
    values.sort_unstable(); // Sắp xếp các giá trị
    values.dedup(); // Loại bỏ các giá trị trùng lặp

    // Xử lý trường hợp đặc biệt A-2-3-4-5
    let has_high_ace = values.contains(&14);
    if has_high_ace {
        values.insert(0, 1); // Thêm Ace với giá trị là 1 vào đầu mảng để giữ thứ tự sau khi sắp xếp
    }

    let mut consecutive_count = 1;
    let mut max_value = 0;

    for i in 0..values.len() - 1 {
        if values[i] + 1 == values[i + 1] {
            consecutive_count += 1;
            if consecutive_count >= 5 {
                max_value = values[i + 1];
            }
        } else {
            consecutive_count = 1;
        }
    }

    if max_value != 0 {
        Some(max_value) // Trả về giá trị cao nhất của Straight
    } else {
        None // Không tìm thấy Straight
    }
}

// Hàm kiểm tra các bộ (Pairs, Three of a Kind, Four of a Kind)
fn check_multiples(cards: &[Card]) -> (Option<u8>, Option<u8>, Vec<u8>, Vec<u8>) {
    let mut counts = [0; 15]; // Mảng đếm từ 2 đến 14
    for card in cards {
        counts[card.value as usize] += 1;
    }

    let mut four = None;
    let mut three = Vec::new();
    let mut pairs = Vec::new();
    let mut singles = Vec::new(); // Track single cards (not part of multiple)

    for (value, &count) in counts.iter().enumerate() {
        match count {
            4 => four = Some(value as u8),
            3 => three.push(value as u8),
            2 => pairs.push(value as u8),
            1 => singles.push(value as u8), // Store single cards
            _ => (),
        }
    }

    // Sắp xếp giảm dần
    pairs.sort_by(|a, b| b.cmp(a));
    three.sort_by(|a, b| b.cmp(a));

    singles.sort_by(|a, b| b.cmp(a));

    if pairs.len() > 2 {
        pairs.truncate(2);
    }

    if (three.is_empty()) {
        return (four, None, pairs, singles);
    } else {
        return (four, Some(three[0] as u8), pairs, singles);
    }
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
        assert_eq!(check_straight(&cards), Some(6));
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
        assert_eq!(check_straight(&cards), Some(7));
    }

    #[test]
    fn test_straight_with_ace_high() {
        let cards = vec![
            Card { value: 10, suit: 0 },
            Card { value: 11, suit: 0 },
            Card { value: 12, suit: 0 },
            Card { value: 13, suit: 0 },
            Card { value: 14, suit: 0 }, // Ace
        ];
        assert_eq!(check_straight(&cards), Some(14));
    }

    #[test]
    fn test_straight_with_ace_low() {
        let cards = vec![
            Card { value: 2, suit: 0 },
            Card { value: 3, suit: 0 },
            Card { value: 4, suit: 0 },
            Card { value: 5, suit: 0 },
            Card { value: 14, suit: 0 }, // Ace
        ];
        assert_eq!(check_straight(&cards), Some(5));
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
        assert_eq!(check_straight(&cards), Some(8));
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
        assert_eq!(check_straight(&cards), Some(6));
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
            (None, None, vec![], vec![10, 8, 6, 4, 2])
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
            (None, None, vec![3], vec![10, 8, 6])
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
        assert_eq!(check_multiples(&cards), (None, None, vec![6, 5], vec![10]));
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
        assert_eq!(check_multiples(&cards), (None, None, vec![7, 6], vec![]));
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
            (None, Some(7), vec![], vec![10, 8])
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
        assert_eq!(check_multiples(&cards), (None, Some(8), vec![], vec![]));
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
        assert_eq!(check_multiples(&cards), (Some(9), None, vec![], vec![10]));
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
        assert_eq!(check_multiples(&cards), (None, Some(2), vec![3], vec![]));
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
            Card { value: 14, suit: 0 }, // Ace
            Card { value: 2, suit: 1 },
        ];
        assert_eq!(check_straight(&cards), Some(14)); // Ace as High
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
        assert_eq!(check_multiples(&cards), (None, Some(8), vec![], vec![]));
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
            (None, None, vec![6], vec![5, 4, 3, 2])
        ); //Only check the pair
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

        // Test comparison when both are Full Houses
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
        let cards = [Card { value: 14, suit: 0 }, Card { value: 3, suit: 0 }]; //Ace
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
        let cards = [Card { value: 14, suit: 0 }, Card { value: 3, suit: 0 }]; //Ace
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
        let cards = [Card { value: 8, suit: 0 }, Card { value: 8, suit: 1 }]; //Ace
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
}

fn parse_cards(input: &str) -> Vec<Card> {
    input
        .split_whitespace()
        .filter_map(|card_str| {
            // Sử dụng filter_map để loại bỏ các giá trị rỗng hoặc không hợp lệ
            if card_str.len() != 2 {
                // Kiểm tra độ dài chuỗi để đảm bảo nó hợp lệ
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

fn print_hand_and_rank(hand: &[Card], board: &[Card]) {
    println!("Hand: {}", hand.iter().map(|card| card.display()).collect::<Vec<_>>().join(" "));
    println!("Board: {}", board.iter().map(|card| card.display()).collect::<Vec<_>>().join(" "));
    println!("Rank: {:?}", evaluate_hand(hand, board));
    println!("----");
}

fn main() {
    let mut rng = thread_rng(); // Create random number generator

    for _ in 0..10 {
        let mut deck = create_deck();
        deck.shuffle(&mut rng);

        let hand = &deck[..2];
        let board = &deck[2..7];

        print_hand_and_rank(hand, board);
    }
    
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

    let hand_vec = parse_cards(hand_input); // Giả sử hàm parse_cards trả về Vec<Card>
    let board_vec = parse_cards(board_input); // Giả sử hàm parse_cards trả về Vec<Card>

    if hand_vec.len() != 2 {
        panic!(
            "Invalid hand length: expected 2 cards, found {}",
            hand_vec.len()
        );
    }

    let hand_array = [hand_vec[0], hand_vec[1]]; // Chuyển đổi Vec<Card> thành [Card; 2]

    for num_players in 2..=9 {
        let (win_rate, _) = simulate_poker_hand(hand_array, board_vec.clone(), num_players);

        println!(
            "Number of players: {}. Simulated Win rate: {:.2}%, EV 1$ bet {:.2}$",
            num_players,
            win_rate * 100.0,
            num_players as f64 * win_rate - 1.0
        );
    }
}
