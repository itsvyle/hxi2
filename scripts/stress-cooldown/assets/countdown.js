// let start = 1756706400;

let start = new Date(2025, 11 - 1, 13).getTime() / 1000;
let deadline = new Date(2025, 11 - 1, 17).getTime() / 1000;

function update() {
    let now = Date.now() / 1000;
    let delta = deadline - now;
    let percentage = (100 * (now - start)) / (deadline - start);

    let days = Math.floor(delta / (3600 * 24));
    let hours = Math.floor((delta % (3600 * 24)) / 3600);
    let minutes = Math.floor((delta % 3600) / 60);
    let seconds = Math.floor(delta % 60);

    document.getElementById("days-value").innerHTML = days;
    document.getElementById("hours-value").innerHTML = hours;
    document.getElementById("minutes-value").innerHTML = minutes;
    document.getElementById("seconds-value").innerHTML = seconds;
}

function updatePercentage() {
    let now = Date.now() / 1000;
    let percentage = Math.min(100, (100 * (now - start)) / (deadline - start));

    document.getElementById("percentage-value").innerHTML =
        percentage.toFixed(7) + " %";
}

update();
updatePercentage();

setInterval(update, 1000);
setInterval(updatePercentage, 10);
