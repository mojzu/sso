export function password_strength(id) {
    var strength = {
        0: "Worst",
        1: "Bad",
        2: "Weak",
        3: "Good",
        4: "Strong",
    };
    var password = document.getElementById(id);
    var meter = document.getElementById("password-strength-meter");
    var warning = document.getElementById("password-strength-warning");

    password.addEventListener("input", function () {
        var result = zxcvbn(password.value);
        meter.value = result.score;
        meter.title = strength[result.score] + " Password";
        warning.innerText = result.feedback.warning;
    });
}
