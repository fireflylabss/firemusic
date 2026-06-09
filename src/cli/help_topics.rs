pub fn discovery() {
    print!(
        "\x1b[1mDISCOVERY\x1b[0m\n\
         \n\
         Search music across providers.\n\
         \n\
         \x1b[1mUSAGE:\x1b[0m\n\
         \x20   -s\n\
         \x20   -s \"query\"\n\
         \x20   -s \"yt:query\"\n\
         \x20   -s \"sc:query\"\n\
         \x20   -s \"tk:query\"\n\
         \n\
         \x1b[1mPROVIDERS:\x1b[0m\n\
         \x20   yt    YouTube\n\
         \x20   sc    SoundCloud\n\
         \x20   tk    TikTok\n\
         \n\
         \x1b[1mENVIRONMENT:\x1b[0m\n\
         \x20   BRAVE_SEARCH_API_KEY\n\
         \x20       Improves TikTok search results\n"
    );
}

pub fn download() {
    print!(
        "\x1b[1mDOWNLOAD\x1b[0m\n\
         \n\
         Download audio or video from supported providers.\n\
         \n\
         \x1b[1mUSAGE:\x1b[0m\n\
         \x20   --download\n\
         \x20   --download=audio <URL>\n\
         \x20   --download=video <URL>\n\
         \n\
         \x1b[1mMODES:\x1b[0m\n\
         \x20   audio     Download MP3\n\
         \x20   video     Download MP4\n"
    );
}

pub fn interface() {
    print!(
        "\x1b[1mINTERFACES\x1b[0m\n\
         \n\
         Terminal user interface.\n\
         \n\
         \x1b[1mUSAGE:\x1b[0m\n\
         \x20   --tui\n\
         \x20   --tui -m ~/Music\n\
         \n\
         \x1b[1mKEYS:\x1b[0m\n\
         \x20   Tab       Switch section\n\
         \x20   F1-F3     Open panel\n"
    );
}

pub fn controls() {
    print!(
        "\x1b[1mPLAYBACK CONTROLS\x1b[0m\n\
         \n\
         \u{250c}{0}\u{252c}{1}\u{2510}\n\
         \u{2502} Key      \u{2502} Action         \u{2502}\n\
         \u{251c}{0}\u{253c}{1}\u{2524}\n\
         \u{2502} Space    \u{2502} Pause / Resume \u{2502}\n\
         \u{2502} \u{2190} \u{2192}      \u{2502} Seek \u{00b1}5s       \u{2502}\n\
         \u{2502} {{ }}      \u{2502} Seek \u{00b1}1m       \u{2502}\n\
         \u{2502} \u{2191} \u{2193}      \u{2502} Volume \u{00b1}       \u{2502}\n\
         \u{2502} + -      \u{2502} Speed \u{00b1}        \u{2502}\n\
         \u{2502} , .      \u{2502} Pitch \u{00b1}        \u{2502}\n\
         \u{2502} 1-9      \u{2502} Jump 10%-90%   \u{2502}\n\
         \u{2502} 0        \u{2502} Reset all      \u{2502}\n\
         \u{251c}{0}\u{253c}{1}\u{2524}\n\
         \u{2502} e        \u{2502} EQ preset      \u{2502}\n\
         \u{2502} E        \u{2502} Manual EQ      \u{2502}\n\
         \u{2502} l        \u{2502} Loop           \u{2502}\n\
         \u{2502} m        \u{2502} Mute           \u{2502}\n\
         \u{251c}{0}\u{253c}{1}\u{2524}\n\
         \u{2502} q, Esc   \u{2502} Quit           \u{2502}\n\
         \u{2514}{0}\u{2534}{1}\u{2518}\n",
        "\u{2500}".repeat(10),
        "\u{2500}".repeat(16),
    );
}
