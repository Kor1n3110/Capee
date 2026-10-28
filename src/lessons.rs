// Original exercises. Methodological references are documented in SOURCES.md.
pub const COURSES: [&str; 4] = [
    "Русский · ЙЦУКЕН",
    "English · QWERTY",
    "Python · QWERTY",
    "C++ · QWERTY",
];
pub const TITLES: [&str; 8] = [
    "Домашний ряд",
    "Верхний ряд",
    "Нижний ряд",
    "Слова и ритм",
    "Цифры и знаки",
    "Короткие фразы",
    "Практика",
    "Итоговый урок",
];
pub const CODE_TITLES: [&str; 8] = [
    "Символы и скобки",
    "Переменные",
    "Выражения",
    "Условия",
    "Циклы",
    "Функции",
    "Коллекции",
    "Итоговая программа",
];
pub fn title(course: usize, lesson: usize) -> &'static str {
    if course < 2 {
        TITLES[lesson]
    } else {
        CODE_TITLES[lesson]
    }
}
pub fn hint(course: usize) -> &'static str {
    if course < 2 {
        "Пальцы на ФЫВА ОЛДЖ / ASDF JKL;. Сначала точность, затем скорость."
    } else {
        "Включите английскую раскладку. Enter — новая строка; Tab — четыре пробела. Отступы важны."
    }
}
pub const TEXTS: [[&str;8];4] = [
["фыва олдж фыва олдж фа ва ол да фы вы жд лы", "йцукен гшщзхъ кен шщз ку цу йц ух зх", "ячсмить бю яч см ит ьб бю мир тир мать", "дом вода рука река город книга утро свет ветер", "123 456 789 0 10 20 30 100 2026 + - = ( ) ! ?", "Мы учимся печатать точно. Руки расслаблены, взгляд направлен на экран.", "Тихое утро наполняет комнату светом. С каждой строкой движения становятся увереннее, а нужные клавиши найти всё легче.", "Скорость приходит с практикой. Старайтесь сохранять ровный ритм и обращайте внимание на точность. Короткие ежедневные занятия помогают закрепить навык слепой печати."],
["asdf jkl; asdf jkl; fj dk sl a; sad fall ask", "qwer tyui op qw er ty ui op quiet power", "zxcv bnm zx cv bn nm mix calm vivid", "home world river morning book light wind city", "123 456 789 0 10 20 30 100 2026 + - = ( ) ! ?", "Keep your hands relaxed. Look at the screen and type with a steady rhythm.", "The morning light fills the room. Every short practice session helps your fingers find the right keys with less effort.", "Accuracy comes before speed. Practice a little every day, keep a comfortable posture, and let a steady rhythm guide your hands. Your next line is another small step forward."],
["() [] {} : , . _ = + - * / # ' \"", "name = \"Ada\"\nage = 25\nis_ready = True", "total = 12 + 8\naverage = total / 4\nmessage = f\"Result: {average}\"", "score = 95\nif score >= 90:\n    print(\"Great work!\")\nelse:\n    print(\"Keep practicing\")", "for number in range(5):\n    print(number)\n\ncount = 0\nwhile count < 3:\n    count += 1", "def greet(name: str) -> str:\n    return f\"Hello, {name}!\"\n\nprint(greet(\"Ada\"))", "scores = [85, 92, 98]\npassed = [s for s in scores if s >= 90]\nuser = {\"name\": \"Ada\", \"scores\": passed}", "def average(values):\n    if not values:\n        return 0\n    return sum(values) / len(values)\n\nif __name__ == \"__main__\":\n    scores = [80, 90, 100]\n    print(f\"Average: {average(scores):.1f}\")"],
["() [] {} ; : , . _ = + - * / < > & # \"", "int age = 25;\ndouble speed = 120.5;\nbool ready = true;", "int total = 12 + 8;\ndouble average = total / 4.0;\nint remainder = total % 3;", "int score = 95;\nif (score >= 90) {\n    std::cout << \"Great work!\";\n} else {\n    std::cout << \"Keep practicing\";\n}", "for (int i = 0; i < 5; ++i) {\n    std::cout << i << '\\n';\n}", "int add(int a, int b) {\n    return a + b;\n}\n\nint result = add(3, 7);", "std::vector<int> scores = {85, 92, 98};\nfor (const auto& score : scores) {\n    std::cout << score << '\\n';\n}", "#include <iostream>\n#include <vector>\n\nint main() {\n    std::vector<int> scores = {80, 90, 100};\n    int total = 0;\n    for (int score : scores) {\n        total += score;\n    }\n    std::cout << total / scores.size() << '\\n';\n    return 0;\n}"]
];

/// Extend a skill block to exactly 100 whitespace-delimited words, retaining code indentation.
pub fn exercise(course: usize, lesson: usize) -> String {
    let seed = TEXTS[course][lesson];
    let separator = if course < 2 { " " } else { "\n\n" };
    let mut text = seed.to_owned();
    while text.split_whitespace().count() < 100 {
        text.push_str(separator);
        text.push_str(seed);
    }
    let mut words = 0;
    let mut in_word = false;
    for (index, c) in text.char_indices() {
        if c.is_whitespace() {
            if in_word && words == 100 {
                text.truncate(index);
                return text;
            }
            in_word = false;
        } else if !in_word {
            words += 1;
            in_word = true;
        }
    }
    text
}

/// Small local PRNG for exercise variety; it is not used for security or credentials.
pub fn next_random(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 0x9e3779b97f4a7c15;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

pub fn endless_block(course: usize, state: &mut u64) -> String {
    let lesson = (next_random(state) % 8) as usize;
    if course >= 2 {
        let number = 10 + next_random(state) % 90;
        let suffix = next_random(state) % 1000;
        return TEXTS[course][lesson]
            .replace("25", &number.to_string())
            .replace("score", &format!("score_{suffix}"))
            .replace("total", &format!("total_{suffix}"));
    }
    if lesson >= 5 {
        return TEXTS[course][lesson].to_string();
    }
    let pool: Vec<_> = TEXTS[course][lesson].split_whitespace().collect();
    (0..24)
        .map(|_| pool[(next_random(state) as usize) % pool.len()])
        .collect::<Vec<_>>()
        .join(" ")
}
