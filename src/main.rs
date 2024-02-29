use rand::seq::SliceRandom; // Sử dụng crate rand để xáo bài

// Định nghĩa cấu trúc cho một lá bài và bàn tay
#[derive(Clone, Copy, Debug, PartialEq)]
struct Card {
    value: u8, // Giá trị của lá bài, 2 đến 14, với 11 là J, 12 là Q, v.v.
    suit: u8,  // Chất của lá bài, có thể định nghĩa từ 0 đến 3
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
    let is_flush = check_flush(&all_cards);
    let straight_high_card = check_straight(&all_cards);
    let (four, three, pairs) = check_multiples(&all_cards);

    // Đánh giá loại bàn tay dựa trên kết quả kiểm tra
    match (is_flush, straight_high_card, four, three, pairs.len()) {
        (true, Some(14), _, _, _) => HandRank::RoyalFlush,
        (true, Some(high_card), _, _, _) => HandRank::StraightFlush(high_card),
        (_, _, Some(card), _, _) => HandRank::FourOfAKind(card),
        (_, _, _, Some(three), pairs_count) if pairs_count > 0 => {
            HandRank::FullHouse(three, pairs[0])
        }
        (true, _, _, _, _) => HandRank::Flush(all_cards.last().unwrap().value),
        (_, Some(high_card), _, _, _) => HandRank::Straight(high_card),
        (_, _, _, Some(card), _) => HandRank::ThreeOfAKind(card),
        (_, _, _, _, 2) => HandRank::TwoPair(pairs[0], pairs[1]),
        (_, _, _, _, 1) => HandRank::OnePair(pairs[0]),
        _ => HandRank::HighCard(all_cards.last().unwrap().value),
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
            HandRank::FourOfAKind(card1) => {
                if let HandRank::FourOfAKind(card2) = hand2 {
                    card1.cmp(&card2) as i32
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
            HandRank::Flush(high_card1) => {
                if let HandRank::Flush(high_card2) = hand2 {
                    high_card1.cmp(&high_card2) as i32
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
            HandRank::TwoPair(high_pair1, low_pair1) => {
                if let HandRank::TwoPair(high_pair2, low_pair2) = hand2 {
                    match high_pair1.cmp(&high_pair2) {
                        std::cmp::Ordering::Equal => low_pair1.cmp(&low_pair2) as i32,
                        other => other as i32,
                    }
                } else {
                    0
                }
            }
            HandRank::OnePair(pair1) => {
                if let HandRank::OnePair(pair2) = hand2 {
                    pair1.cmp(&pair2) as i32
                } else {
                    0
                }
            }
            HandRank::HighCard(high_card1) => {
                if let HandRank::HighCard(high_card2) = hand2 {
                    high_card1.cmp(&high_card2) as i32
                } else {
                    0
                }
            }
        }
    }
}

// Hàm chính để mô phỏng và tính toán xác suất
fn simulate_poker_hand(hand: [Card; 2], board: Vec<Card>, num_players: usize) -> (f64, f64) {
    let mut deck = create_deck();
    let mut wins = 0;
    let mut losses = 0;
    let mut total_simulations = 0;
    let mut losing_hands_info = Vec::new();

    remove_known_cards(&mut deck, &hand, &board);

    for _ in 0..100000 {
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
        let mut strongest_opponent_hand = None;
        let mut strongest_opponent_rank = HandRank::HighCard(0); // Giả sử giá trị thấp nhất

        for other_hand in all_hands.iter().skip(1) {
            let other_rank = evaluate_hand(other_hand, &simulated_board);
            if compare_hands(player_rank, other_rank) != 1 {
                has_worse_hand = true;
                if compare_hands(strongest_opponent_rank, other_rank) == -1 {
                    strongest_opponent_rank = other_rank;
                    strongest_opponent_hand = Some(other_hand);
                }
            }
        }

        if has_worse_hand {
            losses += 1;
            // Chỉ lưu trữ thông tin cho 10 ván thua đầu tiên
            if losses <= 10 {
                if let Some(opponent_hand) = strongest_opponent_hand {
                    let player_hand_str = hand
                        .iter()
                        .map(|card| card.display())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let opponent_hand_str = opponent_hand
                        .iter()
                        .map(|card| card.display())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let board_str = simulated_board
                        .iter()
                        .map(|card| card.display())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let description = format!(
                        "Your hand: {}, Your Rank: {:?}, Opponent hand: {}, Opponent Rank: {:?}, Board: {}",
                        player_hand_str, player_rank, opponent_hand_str, strongest_opponent_rank, board_str
                    );
                    losing_hands_info.push(description);
                }
            }
        } else {
            wins += 1;
        }
        total_simulations += 1;

        deck = create_deck();
        remove_known_cards(&mut deck, &hand, &board);
    }

    // In ra thông tin về 10 bộ bài mạnh nhất mà bạn thua
    for description in losing_hands_info.iter() {
        println!("{}", description);
    }

    let win_rate = wins as f64 / total_simulations as f64;
    let loss_rate = losses as f64 / total_simulations as f64;

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

// Định nghĩa các loại bàn tay
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum HandRank {
    HighCard(u8),
    OnePair(u8),
    TwoPair(u8, u8),
    ThreeOfAKind(u8),
    Straight(u8),
    Flush(u8),
    FullHouse(u8, u8),
    FourOfAKind(u8),
    StraightFlush(u8),
    RoyalFlush,
}

// Hàm kiểm tra Flush
fn check_flush(cards: &[Card]) -> bool {
    // Tạo một mảng để đếm số lượng lá bài cho mỗi chất
    let mut suits = [0; 4]; // Một mảng với 4 phần tử, tương ứng với 4 chất

    // Đếm số lá bài cho mỗi chất
    for card in cards {
        suits[card.suit as usize] += 1;
    }

    // Kiểm tra xem có chất nào có ít nhất 5 lá bài không
    suits.iter().any(|&count| count >= 5)
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
fn check_multiples(cards: &[Card]) -> (Option<u8>, Option<u8>, Vec<u8>) {
    let mut counts = [0; 15]; // Mảng đếm từ 2 đến 14
    for card in cards {
        counts[card.value as usize] += 1;
    }

    let mut four = None;
    let mut three = None;
    let mut pairs = Vec::new();

    for (value, &count) in counts.iter().enumerate() {
        match count {
            4 => four = Some(value as u8),
            3 => three = Some(value as u8),
            2 => pairs.push(value as u8),
            _ => (),
        }
    }

    // Sắp xếp giảm dần và chỉ giữ lại hai giá trị lớn nhất nếu có nhiều hơn hai cặp
    pairs.sort_by(|a, b| b.cmp(a));
    if pairs.len() > 2 {
        pairs.truncate(2); // Chỉ giữ lại hai cặp có giá trị cao nhất
    }

    (four, three, pairs)
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
        assert_eq!(check_multiples(&cards), (None, None, vec![]));
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
        assert_eq!(check_multiples(&cards), (None, None, vec![3]));
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
        assert_eq!(check_multiples(&cards), (None, None, vec![6, 5]));
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
        assert_eq!(check_multiples(&cards), (None, None, vec![7, 6]));
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
        assert_eq!(check_multiples(&cards), (None, Some(7), vec![]));
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
        assert_eq!(check_multiples(&cards), (None, Some(8), vec![]));
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
        assert_eq!(check_multiples(&cards), (Some(9), None, vec![]));
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
        assert_eq!(check_multiples(&cards), (None, Some(2), vec![3]));
    }
}

fn main() {
    // Sử dụng hàm này để chạy mô phỏng
    // let hand = [Card { value: 10, suit: 0 }, Card { value: 11, suit: 2 }]; // Ví dụ: 10♠ và J♦
    let hand = [Card { value: 2, suit: 0 }, Card { value: 2, suit: 1 }];
    let board = vec![
        Card { value: 2, suit: 3 },
        Card { value: 5, suit: 1 },
        Card { value: 7, suit: 2 },
    ]; // Ví dụ: 2♣, 5♥, 7♦
    let num_players = 5; // Số người chơi
    let (win_rate, loss_rate) = simulate_poker_hand(hand, board, num_players);

    println!(
        "Win rate: {:.2}%, Loss rate: {:.2}%",
        win_rate * 100.0,
        loss_rate * 100.0
    );
}
