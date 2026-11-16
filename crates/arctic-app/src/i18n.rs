//! Runtime localisation for the desktop launcher.
//!
//! English is the source language. Keeping the source key in English makes it
//! possible to add more languages without changing persisted data or command
//! line arguments. Text that is not translated yet deliberately falls back to
//! the original English label.

use arctic_core::settings::Language;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

thread_local! {
    static CURRENT: std::cell::Cell<Language> = const { std::cell::Cell::new(Language::English) };
}

// Keep translated strings alive for the whole process because the UI API
// intentionally accepts borrowed labels in many places. Do not apply manual
// Arabic reshaping or reversal here: the current egui text stack handles the
// Unicode glyph shaping/order, and doing it again makes every Arabic label read
// backwards.
static VISUAL_CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();

fn visual_text(text: &str) -> &'static str {
    let cache = VISUAL_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(value) = cache.get(text) {
        return value;
    }
    // Mark embedded Latin runs as left-to-right. Do not reverse them: the
    // renderer already draws their characters in the correct order, while
    // the old manual reversal is what changed `RICO` into a mirrored token.
    let value: &'static str = Box::leak(fix_mixed_rtl(text).into_boxed_str());
    cache.insert(text.to_owned(), value);
    value
}

fn fix_mixed_rtl(text: &str) -> String {
    if !text.chars().any(|c| ('\u{0600}'..='\u{06ff}').contains(&c)) {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut run = String::new();
    let flush = |out: &mut String, run: &mut String| {
        if !run.is_empty() {
            out.push('\u{200e}');
            out.push_str(run);
            out.push('\u{200e}');
            run.clear();
        }
    };
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '/' | '_' | '-' | '@' | '#') {
            run.push(ch);
        } else {
            flush(&mut out, &mut run);
            out.push(ch);
        }
    }
    flush(&mut out, &mut run);
    out
}

/// Set the language used by shared widgets during the current UI frame.
pub fn set_current(language: Language) {
    CURRENT.with(|current| current.set(language));
}

/// Translate a label using the language selected for the current frame.
pub fn current(english: &str) -> &'static str {
    CURRENT.with(|language| text(language.get(), english))
}

/// Translate and interpolate simple display messages.
pub fn format_display(english: &str, args: &[&dyn std::fmt::Display]) -> String {
    let template = current(english);
    let mut out = String::with_capacity(template.len() + args.len() * 8);
    let mut rest = template;
    let mut index = 0;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close_rel) = rest[open..].find('}') else {
            out.push_str(&rest[open..]);
            rest = "";
            break;
        };
        let close = open + close_rel;
        let token = &rest[open + 1..close];
        if token.starts_with('{') {
            out.push('{');
            rest = &rest[open + 2..];
            continue;
        }
        if let Some(arg) = args.get(index) {
            out.push_str(&arg.to_string());
            index += 1;
        } else {
            out.push_str(&rest[open..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Translate a stable UI label. Dynamic messages should use [`text_owned`].
pub fn text(language: Language, english: &str) -> &'static str {
    if language == Language::English {
        return visual_text(english);
    }
    let translated = match english {
        "Play" => "تشغيل",
        "Accounts" => "الحسابات",
        "Instances" => "النسخ",
        "Mods" => "الإضافات",
        "Bedrock" => "بيدروك",
        "Cosmetics" => "المظهر",
        "Friends" => "الأصدقاء",
        "Screenshots" => "لقطات الشاشة",
        "Stats" => "الإحصائيات",
        "Logs" => "السجلات",
        "Settings" => "الإعدادات",
        "About" => "حول البرنامج",
        "Add account" => "إضافة حساب",
        "Add account…" => "إضافة حساب…",
        "Manage accounts" => "إدارة الحسابات",
        "New instance…" => "نسخة جديدة…",
        "New instance" => "نسخة جديدة",
        "Search" => "بحث",
        "Refresh" => "تحديث",
        "Retry" => "إعادة المحاولة",
        "Cancel" => "إلغاء",
        "Close" => "إغلاق",
        "Save" => "حفظ",
        "Delete" => "حذف",
        "Remove" => "إزالة",
        "Open" => "فتح",
        "Import" => "استيراد",
        "Export" => "تصدير",
        "Install" => "تثبيت",
        "Play anyway" => "تشغيل رغم ذلك",
        "Join" => "انضمام",
        "Send" => "إرسال",
        "Done" => "تم",
        "Back" => "رجوع",
        "Continue" => "متابعة",
        "All" => "الكل",
        "Installed" => "مثبّت",
        "Loading…" => "جارٍ التحميل…",
        "Searching Modrinth…" => "جارٍ البحث في Modrinth…",
        "Appearance" => "المظهر",
        "General" => "عام",
        "Game" => "اللعبة",
        "Network & voice" => "الشبكة والصوت",
        "Data" => "البيانات",
        "Language" => "اللغة",
        "Memory" => "الذاكرة",
        "Versions" => "الإصدارات",
        "Game window" => "نافذة اللعبة",
        "Java" => "Java",
        "Network" => "الشبكة",
        "Voice chat" => "المحادثة الصوتية",
        "Updates" => "التحديثات",
        "Launching" => "التشغيل",
        "RICO Client" => "عميل RICO",
        "Shared between instances" => "مشترك بين النسخ",
        "English" => "English",
        "العربية" => "العربية",
        "Changes are saved automatically." => "تُحفظ التغييرات تلقائيًا.",
        "Background" => "الخلفية",
        "RICO scenery" => "منظر RICO",
        "Your screenshot" => "لقطة الشاشة الخاصة بك",
        "Your picture…" => "صورتك…",
        "Animated background" => "خلفية متحركة",
        "Start the launcher maximized" => "بدء البرنامج بحجم مكبّر",
        "Keep RICO in the system tray when closed" => "إبقاء RICO في منطقة الإشعارات عند الإغلاق",
        "Close Minecraft when RICO quits" => "إغلاق Minecraft عند الخروج من RICO",
        "Ask before launching Minecraft to join a server" => {
            "السؤال قبل تشغيل Minecraft للانضمام إلى خادم"
        }
        "Maximum" => "الحد الأقصى",
        "Minimum" => "الحد الأدنى",
        "Show snapshots" => "إظهار الإصدارات التجريبية",
        "Show old alpha and beta versions" => "إظهار إصدارات Alpha وBeta القديمة",
        "Resolution" => "الدقة",
        "Presets" => "إعدادات جاهزة",
        "Start in fullscreen" => "البدء بملء الشاشة",
        "When the game starts" => "عند بدء اللعبة",
        "Keep launcher open" => "إبقاء البرنامج مفتوحًا",
        "Minimize launcher" => "تصغير البرنامج",
        "Channel" => "القناة",
        "Stable" => "مستقرة",
        "Beta" => "تجريبية",
        "Check for updates on start" => "البحث عن التحديثات عند البدء",
        "Install updates automatically" => "تثبيت التحديثات تلقائيًا",
        "Restore defaults" => "استعادة الإعدادات الافتراضية",
        "Name" => "الاسم",
        "Path" => "المسار",
        "Type" => "النوع",
        "Address" => "العنوان",
        "Port" => "المنفذ",
        "Password" => "كلمة المرور",
        "User name" => "اسم المستخدم",
        "Version" => "الإصدار",
        "On" => "مفعّل",
        "Off" => "معطّل",
        "Auto" => "تلقائي",
        "New" => "جديد",
        "Latest" => "الأحدث",
        "Older" => "الأقدم",
        "Worlds" => "العوالم",
        "Servers" => "الخوادم",
        "Packs" => "الحزم",
        "Browse mods" => "تصفح الإضافات",
        "All instances" => "كل النسخ",
        "Create instance" => "إنشاء نسخة",
        "127.0.0.1 or proxy.example.com" => "127.0.0.1 أو proxy.example.com",
        "About this project" => "حول هذا المشروع",
        "Add an instance, HUD layout or crosshair someone shared" => {
            "أضف نسخة, أو تخطيط HUD أو مؤشر تصويب شاركه أحدهم"
        }
        "Animated capes and sculpted cosmetics stand still here. Only your own view changes; what you and others wear stays as chosen." => {
            "تبقى العباءات المتحركة وعناصر الزينة المجسّمة ثابتة هنا. يتغير ما تراه أنت فقط; أما ما ترتديه أنت والآخرون فيبقى كما اختير."
        }
        "Another tip" => "نصيحة أخرى",
        "Apply to" => "تطبيق على",
        "Back to preview · Esc" => "العودة إلى المعاينة · Esc",
        "Bring instances, worlds and HUD setups over from TLauncher, CurseForge, Prism, Lunar…" => {
            "انقل النسخ, والعوالم وإعدادات HUD من TLauncher, CurseForge, Prism, Lunar…"
        }
        "Choose what friends see: online status, the server you play on, invites, your name" => {
            "اختر ما يراه الأصدقاء: حالة اتصالك, الخادم الذي تلعب عليه, الدعوات, اسمك"
        }
        "Click to enlarge" => "انقر للتكبير",
        "Contact for crashes" => "وسيلة التواصل بشأن الأعطال",
        "Copy a player's skin by name" => "نسخ مظهر لاعب باستخدام اسمه",
        "Crash reports" => "تقارير الأعطال",
        "Direct join: play.example.net" => "انضمام مباشر: play.example.net",
        "Discord name or e-mail (optional)" => "اسم Discord أو البريد الإلكتروني (اختياري)",
        "Discord name or email, if you want an answer" => {
            "اسم Discord أو البريد الإلكتروني, إذا أردت ردًا"
        }
        "Double-click to rename" => "انقر مرتين لإعادة التسمية",
        "Extra JVM arguments" => "وسائط JVM إضافية",
        "Favorite" => "مفضّل",
        "Filter…" => "تصفية…",
        "Friend code or Microsoft name" => "رمز الصديق أو اسم Microsoft",
        "Has a Microsoft account" => "لديه حساب Microsoft",
        "Installing…" => "جارٍ التثبيت…",
        "Instance settings" => "إعدادات النسخة",
        "Its friends move here too" => "ينتقل أصدقاؤه إلى هنا أيضًا",
        "Laptops with two graphics cards: Minecraft runs on the fast one instead of the built-in one. A choice you made yourself (Windows' graphics settings, or PRIME variables on Linux) is kept." => {
            "الحواسيب المحمولة ذات بطاقتي رسوميات: يعمل Minecraft على البطاقة الأسرع بدلًا من المدمجة. يُحافَظ على اختيارك اليدوي (إعدادات الرسوميات في Windows, أو متغيرات PRIME في Linux)."
        }
        "Loading project from Modrinth…" => "جارٍ تحميل المشروع من Modrinth…",
        "Look up another name…" => "البحث عن اسم آخر…",
        "Manage profiles…" => "إدارة ملفات التعريف…",
        "Microphone" => "الميكروفون",
        "New group…" => "مجموعة جديدة…",
        "New profile…" => "ملف تعريف جديد…",
        "New versions download by themselves and are used the next time you open the launcher." => {
            "تُنزَّل الإصدارات الجديدة تلقائيًا وتُستخدم في المرة القادمة التي تفتح فيها المشغّل."
        }
        "Off: friends see only your profile name, so alts stay private." => {
            "معطّل: لا يرى الأصدقاء سوى اسم ملفك التعريفي, فتظل حساباتك البديلة خاصة."
        }
        "On by default: restart into an installed update while hidden in the tray with no game running. Off: use the update next time you open the launcher, or choose Restart now here." => {
            "مفعّل افتراضيًا: يُعاد تشغيل المشغّل لتطبيق التحديث المثبّت عندما يكون مخفيًا في منطقة الإشعارات ولا توجد لعبة قيد التشغيل. معطّل: يُطبَّق التحديث عند فتح المشغّل في المرة القادمة, أو باختيار إعادة التشغيل الآن هنا."
        }
        "On: games start with the RICO Client (HUD, zoom, cosmetics, replays and more). Off: no game does, whatever its own switch says; the mod is taken out of the instance's mods folder." => {
            "مفعّل: تبدأ الألعاب باستخدام RICO Client (HUD, التقريب, عناصر الزينة, الإعادات والمزيد). معطّل: لا تستخدمه أي لعبة, بغضّ النظر عن حالة مفتاحها الخاص; ويُزال المود من مجلد mods الخاص بالنسخة."
        }
        "Optional, one line" => "اختياري, سطر واحد",
        "Optional. Sent with a crash report so we can write back to you: a Discord name or an e-mail address. Leave it empty to stay anonymous." => {
            "اختياري. يُرسل مع تقرير العطل لنتمكن من الرد عليك: اسم Discord أو عنوان بريد إلكتروني. اتركه فارغًا لتبقى مجهول الهوية."
        }
        "Path to javaw.exe" => "مسار javaw.exe",
        "Preview project" => "معاينة المشروع",
        "Puts these settings into that instance now (the rest of its settings stay)" => {
            "يطبّق هذه الإعدادات على تلك النسخة الآن (مع إبقاء بقية إعداداتها)"
        }
        "Reach Mojang's services through the proxy" => "الوصول إلى خدمات Mojang عبر الوكيل",
        "Remove account" => "إزالة الحساب",
        "Remove from my capes" => "إزالة من عباءاتي",
        "Removed instances, to bring back or delete for good" => {
            "النسخ المُزالة, لاستعادتها أو حذفها نهائيًا"
        }
        "Rename…" => "إعادة التسمية…",
        "Report this skin" => "الإبلاغ عن هذا المظهر",
        "Right-click to take it off this list" => "انقر بزر الفأرة الأيمن لإزالته من هذه القائمة",
        "Search installed mods…" => "البحث في المودات المثبّتة…",
        "Search modpacks on Modrinth" => "البحث عن حزم مودات على Modrinth",
        "Search mods on Modrinth" => "البحث عن مودات على Modrinth",
        "Search names and tags" => "البحث في الأسماء والوسوم",
        "Search skins or creators" => "البحث عن مظاهر أو منشئيها",
        "Search versions" => "البحث عن إصدارات",
        "Send this" => "إرسال هذا",
        "Send to…" => "إرسال إلى…",
        "Sensitivity" => "الحساسية",
        "Set as Minecraft skin" => "تعيين كمظهر في Minecraft",
        "Skin name" => "اسم المظهر",
        "Steve" => "Steve",
        "Switch account" => "تبديل الحساب",
        "Switch instance" => "تبديل النسخة",
        "Switch profile" => "تبديل ملف التعريف",
        "Talk" => "التحدث",
        "This profile can no longer be restored with that Discord account" => {
            "لم يعد بالإمكان استعادة ملف التعريف هذا باستخدام حساب Discord ذاك"
        }
        "Turn it off for pure vanilla or just your own mods. You can change it later in the instance's page." => {
            "عطّله لتشغيل اللعبة الأصلية بلا مودات أو بموداتك فقط. يمكنك تغيير ذلك لاحقًا في صفحة النسخة."
        }
        "Use your own cape image: 64×32 up to 1024×512; stack up to 32 frames (12288 pixels tall) to animate it, at 8 frames a second. It stays here to wear again." => {
            "استخدم صورة عباءتك الخاصة: من 64×32 إلى 1024×512; رتّب حتى 32 إطارًا فوق بعضها (بارتفاع 12288 بكسل) لتحريكها, بمعدل 8 إطارات في الثانية. تبقى هنا لارتدائها مجددًا."
        }
        "Vanilla: adds Sodium, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, Particle Core, BadOptimizations, ModernFix and Polarium. Other instances have their own switch." => {
            "النسخة الأصلية: يضيف Sodium, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, Particle Core, BadOptimizations, ModernFix وPolarium. للنسخ الأخرى مفتاح خاص بها."
        }
        "Volume" => "مستوى الصوت",
        "When the game crashes, RICO can send its log so the crash can be fixed. Your name, folders and sign-in details are taken out first." => {
            "عند تعطل اللعبة, يمكن لـRICO إرسال سجلّها للمساعدة في إصلاح العطل. تُحذف أولًا معلومات اسمك, ومجلداتك وبيانات تسجيل دخولك."
        }
        "abcd-efgh" => "abcd-efgh",
        "arctic-xxxx-xxxx-…" => "arctic-xxxx-xxxx-…",
        "e.g. -XX:+UseZGC" => "مثلًا: -XX:+UseZGC",
        "e.g. 25565" => "مثلًا: 25565",
        "e.g. Speedruns, Family, Testing" => "مثلًا: تحديات الإنهاء السريع, العائلة, الاختبار",
        "optional" => "اختياري",
        "play.example.net" => "play.example.net",
        "play.example.net:19132" => "play.example.net:19132",
        "smp, pvp, minigames (up to 5)" => "smp, pvp, ألعاب مصغّرة (حتى 5)",
        "xxxx-xxxx-xxxx-xxxx" => "xxxx-xxxx-xxxx-xxxx",
        "×" => "×",
        // Settings and statistics pages.
        "Where RICO keeps its files" => "مكان حفظ ملفات RICO",
        "Open folder" => "فتح المجلد",
        "Change folder…" => "تغيير المجلد…",
        "Game files, Java, instances (worlds, mods, packs), accounts and settings are kept in:" => {
            "تُحفظ ملفات اللعبة وJava والنسخ (العوالم والإضافات والحزم) والحسابات والإعدادات في:"
        }
        "Portable mode: portable.txt is next to the launcher, so everything stays in the data folder beside it." => {
            "الوضع المحمول: يوجد portable.txt بجانب المشغّل، لذلك تبقى كل البيانات في مجلد البيانات بجانبه."
        }
        "Another drive or a bigger disk? Change the folder and everything moves there." => {
            "هل تريد قرصًا آخر أو مساحة أكبر؟ غيّر المجلد وستُنقل كل الملفات إليه."
        }
        "Game folder changed" => "تم تغيير مجلد اللعبة",
        "Couldn't change the game folder" => "تعذّر تغيير مجلد اللعبة",
        "Moving RICO's files…" => "جارٍ نقل ملفات RICO…",
        "RICO restarts when it's done." => "سيُعاد تشغيل RICO عند الانتهاء.",
        "The files couldn't be moved" => "تعذّر نقل الملفات",
        "OK" => "موافق",
        "Can't use this folder" => "لا يمكن استخدام هذا المجلد",
        "Move RICO's files here?" => "هل تريد نقل ملفات RICO إلى هنا؟",
        "Close Minecraft first." => "أغلق Minecraft أولًا.",
        "This folder already has RICO's files" => "يحتوي هذا المجلد على ملفات RICO بالفعل",
        "This folder has other things in it" => "يحتوي هذا المجلد على ملفات أخرى",
        "Move files" => "نقل الملفات",
        "Use these files" => "استخدام هذه الملفات",
        "Use that folder" => "استخدام ذلك المجلد",
        "Pick another" => "اختيار مجلد آخر",
        "Game folder" => "مجلد اللعبة",
        "Move files there" => "نقل الملفات إلى هناك",
        "Use it as it is" => "استخدامه كما هو",
        "Share: mods, HUD, crosshair" => "مشاركة الإضافات وHUD ومؤشر التصويب",
        "HUD layout" => "تخطيط HUD",
        "Crosshair" => "مؤشر التصويب",
        "All client settings" => "كل إعدادات العميل",
        "Share your profile" => "مشاركة ملفك الشخصي",
        "Get a code" => "الحصول على رمز",
        "Add an account first; codes are made by signed-in players." => {
            "أضف حسابًا أولًا؛ تُنشأ الرموز بواسطة اللاعبين المسجلين دخولًا."
        }
        "Copy as text" => "نسخ كنص",
        "Save file…" => "حفظ الملف…",
        "Making a code…" => "جارٍ إنشاء الرمز…",
        "Copy code" => "نسخ الرمز",
        "Code copied" => "تم نسخ الرمز",
        "Looking it up…" => "جارٍ البحث عنه…",
        "Open file…" => "فتح ملف…",
        "Importing…" => "جارٍ الاستيراد…",
        "Could not save" => "تعذّر الحفظ",
        "Pick what to bring in" => "اختر ما تريد استيراده",
        "Close that game first; it saves its own settings when it quits." => {
            "أغلق اللعبة أولًا؛ فهي تحفظ إعداداتها عند الخروج."
        }
        "Tick at least one thing to import." => "حدد عنصرًا واحدًا على الأقل لاستيراده.",
        "Close your games first." => "أغلق ألعابك أولًا.",
        "Suggest a feature or report a bug" => "اقتراح ميزة أو الإبلاغ عن خطأ",
        "It goes straight to the RICO team. Nothing is sent until you press Send." => {
            "سيصل مباشرة إلى فريق RICO. لن يُرسل شيء حتى تضغط إرسال."
        }
        "Idea" => "فكرة",
        "Bug" => "خطأ",
        "What went wrong, and what were you doing?" => "ما الذي حدث خطأ، وماذا كنت تفعل؟",
        "What would you like RICO to do?" => "ماذا تريد من RICO أن يفعل؟",
        "HOW TO REACH YOU (OPTIONAL)" => "كيفية التواصل معك (اختياري)",
        "Attach the launcher's log" => "إرفاق سجل المشغّل",
        "Attach the last game's log" => "إرفاق سجل آخر لعبة",
        "Logs help find what went wrong. Your user name, folders, emails, IP addresses and sign-in tokens are taken out before sending." => {
            "تساعد السجلات في معرفة الخطأ. تُحذف اسم المستخدم والمجلدات والبريد وعناوين IP ورموز تسجيل الدخول قبل الإرسال."
        }
        "Need an answer sooner? Our Discord is quick to help." => {
            "تحتاج إلى إجابة أسرع؟ يساعدك Discord لدينا بسرعة."
        }
        "Join the Discord" => "الانضمام إلى Discord",
        "Sending…" => "جارٍ الإرسال…",
        "Sent, thank you" => "تم الإرسال، شكرًا لك",
        "We read every one." => "نقرأ كل رسالة.",
        "Couldn't send it" => "تعذّر الإرسال",
        "Play together" => "اللعب معًا",
        "Invite friends into your world. No server and no port forwarding." => {
            "ادعُ أصدقاءك إلى عالمك. لا تحتاج إلى خادم أو إعادة توجيه منافذ."
        }
        "Host a world" => "استضافة عالم",
        "Your invite code" => "رمز دعوتك",
        "Invite code copied" => "تم نسخ رمز الدعوة",
        "Waiting for a world. In game, press Esc and choose Open to LAN." => {
            "بانتظار عالم. داخل اللعبة اضغط Esc واختر الفتح للشبكة المحلية."
        }
        "No friends connected yet" => "لم يتصل أي أصدقاء بعد",
        "Stop sharing" => "إيقاف المشاركة",
        "Starting…" => "جارٍ البدء…",
        "Start a world in singleplayer." => "ابدأ عالمًا في اللعب الفردي.",
        "Press Esc, choose Open to LAN, then Start LAN World." => {
            "اضغط Esc، واختر الفتح للشبكة المحلية، ثم ابدأ عالم LAN."
        }
        "Send your friends the invite code." => "أرسل رمز الدعوة إلى أصدقائك.",
        "Start sharing" => "بدء المشاركة",
        "Share a specific port" => "مشاركة منفذ محدد",
        "Leave empty to find the LAN world automatically." => {
            "اتركه فارغًا للعثور على عالم الشبكة المحلية تلقائيًا."
        }
        "Join a friend" => "الانضمام إلى صديق",
        "Start Minecraft (same version as your friend)." => "ابدأ Minecraft (بنفس إصدار صديقك).",
        "Open Multiplayer. The world is listed under the LAN section." => {
            "افتح اللعب الجماعي. سيظهر العالم ضمن قسم LAN."
        }
        "Or Direct Connect to" => "أو الاتصال المباشر بـ",
        "Copy address" => "نسخ العنوان",
        "Launch Minecraft" => "تشغيل Minecraft",
        "Leave" => "مغادرة",
        "Connecting to your friend…" => "جارٍ الاتصال بصديقك…",
        "Paste the invite code your friend sent you." => "ألصق رمز الدعوة الذي أرسله صديقك.",
        "Hosting a world" => "يستضيف عالمًا",
        "Playing with friends" => "يلعب مع الأصدقاء",
        "Connected" => "متصل",
        "The world now shows up in Multiplayer." => "يظهر العالم الآن في اللعب الجماعي.",
        "Voice chat on" => "المحادثة الصوتية مفعّلة",
        "When I speak" => "عند تحدثي",
        "Push to talk" => "اضغط للتحدث",
        "Hold V in game to talk (change it in the Arctic menu → Voice)." => {
            "اضغط باستمرار على V داخل اللعبة للتحدث (غيّره من قائمة Arctic ← الصوت)."
        }
        "Lower picks up quieter speech (and more background noise)." => {
            "يلتقط المستوى الأقل الكلام الهادئ (وضوضاء خلفية أكثر)."
        }
        "Microphone device" => "جهاز الميكروفون",
        "Output device" => "جهاز الإخراج",
        "Refresh devices" => "تحديث الأجهزة",
        "Only friends" => "الأصدقاء فقط",
        "Hear and be heard only by your friends." => "اسمع أصدقاءك ولا يسمعك إلا أصدقاؤك.",
        "Also talk with Simple Voice Chat players" => "التحدث أيضًا مع لاعبي Simple Voice Chat",
        "Unmute all" => "إلغاء كتم الجميع",
        "System default" => "الإعداد الافتراضي للنظام",
        "Update required" => "تحديث مطلوب",
        "Update & Restart" => "تحديث وإعادة التشغيل",
        "Exit" => "خروج",
        "Release notes" => "ملاحظات الإصدار",
        "Later" => "لاحقًا",
        "Restart now" => "إعادة التشغيل الآن",
        "Update installed: it's used next time you open the launcher." => {
            "تم تثبيت التحديث؛ سيُستخدم عند فتح المشغّل التالي."
        }
        "Update failed: {}" => "فشل التحديث: {}",
        "Could not restart" => "تعذّرت إعادة التشغيل",
        "Got it" => "فهمت",
        "Don't show this again" => "عدم إظهار هذا مجددًا",
        "What's new" => "ما الجديد",
        "What's fixed" => "ما تم إصلاحه",
        "Click an account to play as it. Sign-ins stay on this PC, encrypted for your user." => {
            "انقر على حساب للعب به. تبقى عمليات تسجيل الدخول على هذا الجهاز ومشفّرة لمستخدمك."
        }
        "Microsoft" => "Microsoft",
        "Offline" => "غير متصل",
        "Playing as {}" => "اللعب باسم {}",
        "Pick an instance and jump in." => "اختر نسخة وابدأ اللعب.",
        "VERSION" => "الإصدار",
        "ALSO RUNNING" => "تعمل أيضًا",
        "View its log" => "عرض سجلها",
        "Force close this game" => "إغلاق هذه اللعبة بالقوة",
        "Played {} · {} session{} · last {}" => "لُعبت {} · {} جلسة{} · آخر نشاط {}",
        "Default instance" => "النسخة الافتراضية",
        "Also running" => "تعمل أيضًا",
        "Not installed" => "غير مثبت",
        "Star a version to keep it here." => "أضف نجمة لإصدار ليبقى هنا.",
        "No matching versions" => "لا توجد إصدارات مطابقة",
        "LATEST" => "الأحدث",
        "Open game folder" => "فتح مجلد اللعبة",
        "Open last game log" => "فتح آخر سجل للعبة",
        "Force close Minecraft" => "إغلاق Minecraft بالقوة",
        "A required launcher update must be installed first." => {
            "يجب تثبيت تحديث مطلوب للمشغّل أولًا."
        }
        "Add an account to play." => "أضف حسابًا للعب.",
        "Loading versions…" => "جارٍ تحميل الإصدارات…",
        "Minecraft is running. Have fun!" => "Minecraft تعمل الآن. استمتع!",
        "Ready to play." => "جاهز للعب.",
        "This version will be downloaded when you press Play." => {
            "سيتم تنزيل هذا الإصدار عند الضغط على تشغيل."
        }
        "Play this world (1.20+)" => "تشغيل هذا العالم (1.20+)",
        "Do you want to launch Minecraft to play this world?" => {
            "هل تريد تشغيل Minecraft للعب في هذا العالم؟"
        }
        "Just now" => "الآن",
        "Yesterday" => "أمس",
        "Browse and manage mods for your Minecraft instances." => {
            "تصفّح إضافات نسخ Minecraft وأدرها."
        }
        "Choose an instance" => "اختر نسخة",
        "Instance" => "النسخة",
        "RICO Library" => "مكتبة RICO",
        "Browse Modrinth" => "تصفّح Modrinth",
        "This instance has no mod loader" => "هذه النسخة لا تحتوي على محمّل إضافات",
        "Choose a Fabric, Quilt, Forge or NeoForge instance to install and manage mods." => {
            "اختر نسخة Fabric أو Quilt أو Forge أو NeoForge لتثبيت الإضافات وإدارتها."
        }
        "Create a modded instance" => "إنشاء نسخة مع إضافات",
        "Modpacks" => "حزم الإضافات",
        "Install a whole modpack from Modrinth in one click, or drop a .mrpack file anywhere on this page." => {
            "ثبّت حزمة إضافات كاملة من Modrinth بنقرة واحدة، أو أسقط ملف ‎.mrpack في أي مكان بهذه الصفحة."
        }
        "Browse modpacks" => "تصفّح حزم الإضافات",
        "Loading mods…" => "جارٍ تحميل الإضافات…",
        "Import .jar files" => "استيراد ملفات ‎.jar",
        "Look for updates" => "البحث عن تحديثات",
        "Clear" => "مسح",
        "Could not refresh mods: {}" => "تعذّر تحديث الإضافات: {}",
        "No mods yet" => "لا توجد إضافات بعد",
        "Browse Modrinth, import .jar files, or drop them here." => {
            "تصفّح Modrinth أو استورد ملفات ‎.jar أو أسقطها هنا."
        }
        "No installed mods match your search." => "لا توجد إضافات مثبّتة تطابق بحثك.",
        "Looking for newer builds…" => "جارٍ البحث عن إصدارات أحدث…",
        "Updating mods…" => "جارٍ تحديث الإضافات…",
        "All your mods are up to date." => "جميع إضافاتك محدّثة.",
        "Update all" => "تحديث الكل",
        "Not now" => "ليس الآن",
        "Drop to add these mods" => "أسقط لإضافة هذه الإضافات",
        "Relevance" => "الصلة",
        "Most downloads" => "الأكثر تنزيلًا",
        "Recently updated" => "المحدّثة مؤخرًا",
        "Newest" => "الأحدث",
        "Search failed: {}" => "فشل البحث: {}",
        "No mods found." => "لم يُعثر على إضافات.",
        "No modpacks found." => "لم يُعثر على حزم إضافات.",
        "Starting the download" => "بدء التنزيل",
        "Downloads" => "التنزيلات",
        "Hide versions" => "إخفاء الإصدارات",
        "This modpack has no installable versions." => {
            "لا تحتوي حزمة الإضافات هذه على إصدارات قابلة للتثبيت."
        }
        "Could not load the versions: {}" => "تعذّر تحميل الإصدارات: {}",
        "Installing modpack…" => "جارٍ تثبيت حزمة الإضافات…",
        "Curated performance and graphics mods. Files are installed from Modrinth." => {
            "إضافات أداء ورسوميات منتقاة. تُثبّت الملفات من Modrinth."
        }
        "Downloading compatible builds from Modrinth…" => {
            "جارٍ تنزيل الإصدارات المتوافقة من Modrinth…"
        }
        "Favorites" => "المفضلة",
        "Browse" => "تصفّح",
        "Packs & shaders" => "الحزم والتظليل",
        "Remove instance" => "إزالة النسخة",
        "Keep it" => "الاحتفاظ بها",
        "Yes, remove" => "نعم، أزلها",
        "Duplicate instance" => "تكرار النسخة",
        "Duplicate" => "تكرار",
        "Extras" => "إضافات أخرى",
        "Performance" => "الأداء",
        "Share settings with my other instances" => "مشاركة الإعدادات مع النسخ الأخرى",
        "Boost FPS with performance mods" => "زيادة FPS بإضافات الأداء",
        "Could not update mods" => "تعذّر تحديث الإضافات",
        "Could not install mod" => "تعذّر تثبيت الإضافة",
        "Already installed" => "مثبّتة بالفعل",
        "Could not import mods" => "تعذّر استيراد الإضافات",
        "No new mods added" => "لم تُضف إضافات جديدة",
        "Could not remove mod" => "تعذّرت إزالة الإضافة",
        "Could not change mod" => "تعذّر تغيير الإضافة",
        "Could not save instance" => "تعذّر حفظ النسخة",
        "Instance removed" => "تمت إزالة النسخة",
        "Open instance folder" => "فتح مجلد النسخة",
        "Replays" => "الإعادات",
        "Everyone can find and use it in the RICO gallery" => {
            "يمكن للجميع العثور عليه واستخدامه في معرض RICO"
        }
        "Everyone sees this name in the gallery. Pick one that says what the skin is, like \"Ice Knight\"." => {
            "سيرى الجميع هذا الاسم في المعرض. اختر اسمًا يوضح المظهر، مثل «فارس الجليد»."
        }
        "by {} · used {} times" => "بواسطة {} · استُخدم {} مرة",
        "{} skins. Click one to wear it (it's saved to your library). Right-click for more." => {
            "{} مظهر. انقر على أحدها لارتدائه (وسيُحفظ في مكتبتك). انقر بزر الفأرة الأيمن للمزيد."
        }
        "RICO players see an RICO cape; without one, your Minecraft cape shows." => {
            "يرى لاعبو RICO عباءة RICO؛ وإن لم توجد، فستظهر عباءة Minecraft الخاصة بك."
        }
        "Fit {}" => "ضبط {}",
        "{} on the other shoulder" => "{} على الكتف الآخر",
        "RICO players see the skin you pick here; others see your Minecraft skin." => {
            "يرى لاعبو RICO المظهر الذي تختاره هنا؛ بينما يرى الآخرون مظهرك في Minecraft."
        }
        "Skins RICO players have shared, free for anyone. Click one to wear it." => {
            "مظاهر شاركها لاعبو RICO ومتاحة للجميع مجانًا. انقر على أحدها لارتدائه."
        }
        "Only Minecraft skin PNGs are accepted: 64×64, or legacy 64×32." => {
            "تُقبل ملفات PNG لمظاهر Minecraft فقط: 64×64 أو الصيغة القديمة 64×32."
        }
        "OptiFine capes · from your accounts" => "عباءات OptiFine — من حساباتك",
        "No OptiFine cape found for your accounts. OptiFine keeps capes by player name: if yours is under another name, look it up above. RICO also shows other players' OptiFine capes in game." => {
            "لم يُعثر على عباءة OptiFine لحساباتك. يحتفظ OptiFine بالعباءات حسب اسم اللاعب؛ إذا كانت تحت اسم آخر، فابحث عنها أعلاه. يعرض RICO أيضًا عباءات OptiFine للاعبين الآخرين داخل اللعبة."
        }
        "RICO capes are loading." => "جارٍ تحميل عباءات RICO.",
        "Explore the collection below. Add an account to wear capes and cosmetics." => {
            "استكشف المجموعة أدناه. أضف حسابًا لارتداء العباءات والعناصر التجميلية."
        }
        "Collection unavailable · Retry" => "المجموعة غير متاحة — إعادة المحاولة",
        "Can't reach the RICO looks server right now." => "تعذّر الوصول إلى خادم مظاهر RICO حاليًا.",
        "Signing in to RICO — you can look around while it loads." => {
            "جارٍ تسجيل الدخول إلى RICO — يمكنك تصفّح المجموعة أثناء التحميل."
        }
        "Skins, capes and cosmetics every RICO player sees, on any account." => {
            "مظاهر وعباءات وعناصر تجميلية يراها كل لاعب RICO على أي حساب."
        }
        "On. Turn off to show this cape's still image to everyone." => {
            "مفعّل. أوقفه لعرض الصورة الثابتة لهذه العباءة للجميع."
        }
        "Off: everyone sees this cape's still image. Remembered for each cape." => {
            "متوقف: سيرى الجميع الصورة الثابتة لهذه العباءة. يُحفظ الإعداد لكل عباءة."
        }
        "Keep a copy of the skin you're wearing" => "الاحتفاظ بنسخة من المظهر الذي ترتديه",
        "Go back to the default Minecraft skin (seen by everyone)" => {
            "العودة إلى مظهر Minecraft الافتراضي (الذي يراه الجميع)"
        }
        "LOOKS" => "المظاهر",
        "COSMETICS" => "العناصر التجميلية",
        "Skin" => "المظهر",
        "Cape" => "العباءة",
        "Head" => "الرأس",
        "Face" => "الوجه",
        "Shoulders" => "الكتفان",
        "Arms" => "الذراعان",
        "Body" => "الجسم",
        "Take it off" => "خلعه",
        "No skin yet" => "لا يوجد مظهر بعد",
        "Trying on" => "تجربة الارتداء",
        "Display model" => "نموذج العرض",
        "Remove all" => "إزالة الكل",
        "Save to library" => "حفظ في المكتبة",
        "Reset Minecraft skin" => "إعادة ضبط مظهر Minecraft",
        "Saving…" => "جارٍ الحفظ…",
        "Saved" => "تم الحفظ",
        "Type their friend code or their Microsoft account's name. They need to use Arctic too." => {
            "اكتب رمز الصديق أو اسم حساب Microsoft الخاص به. يجب أن يستخدم Arctic أيضًا."
        }
        "Your friend code: give it to people so they can add you" => {
            "رمز صداقتك: أعطه للآخرين ليتمكنوا من إضافتك"
        }
        "Link your Discord to get this profile back, with its friends, on another PC." => {
            "اربط Discord لاستعادة هذا الملف مع أصدقائه على جهاز آخر."
        }
        "Unlink (it keeps no friends)" => "إلغاء الربط (لن يحتفظ بأي أصدقاء)",
        "DISCORD" => "Discord",
        "ACCOUNTS ON THIS PROFILE" => "الحسابات في ملف التعريف هذا",
        "RECOVERY CODE" => "رمز الاسترداد",
        "Or, if you linked your Discord:" => "أو، إذا ربطت Discord:",
        "Friends follow you on {} too. Skip to keep it separate." => {
            "سيتبعك الأصدقاء على {} أيضًا. تخطَّ ذلك لإبقائه منفصلًا."
        }
        "their world" => "عالمهم",
        "in game" => "داخل اللعبة",
        "online" => "متصل",
        "offline" => "غير متصل",
        "playing on" => "يلعب على",
        "Invite to your shared world" => "دعوة إلى عالمك المشترك",
        "Start and join" => "البدء والانضمام",
        "Start the game in a duel arena and invite {}" => "ابدأ اللعبة في ساحة مبارزة وادعُ {}",
        "No friends yet. Add someone by their friend code (or a Microsoft account's name); they need to use Arctic too." => {
            "لا يوجد أصدقاء بعد. أضف شخصًا باستخدام رمز الصديق أو اسم حساب Microsoft؛ ويجب أن يستخدم Arctic أيضًا."
        }
        "Friends couldn't be loaded: {}" => "تعذّر تحميل الأصدقاء: {}",
        "Link {} to your friends profile?" => "ربط {} بملف أصدقائك؟",
        "to {}" => "إلى {}",
        "{} invited you to {}" => "دعاك {} إلى {}",
        "{} wants to be friends" => "يريد {} أن يصبح صديقك",
        "Asked {}" => "تم إرسال طلب إلى {}",
        "Linked to {}" => "مرتبط بـ {}",
        "as {}" => "باسم {}",
        "code {}" => "الرمز {}",
        "Chat ({})" => "محادثة ({})",
        "Invite to {}" => "دعوة إلى {}",
        "Start and join {}" => "البدء والانضمام إلى {}",
        "Skins" => "المظاهر",
        "Skin gallery" => "معرض المظاهر",
        "Capes" => "العباءات",
        "Add skin from your files" => "إضافة مظهر من ملفاتك",
        "Open skins folder" => "فتح مجلد المظاهر",
        "Get" => "جلب",
        "Minecraft skin" => "مظهر Minecraft",
        "Add skin…" => "إضافة مظهر…",
        "Wearing" => "يُرتدى الآن",
        "Your own Minecraft skin, for RICO players too" => {
            "مظهرك في Minecraft، ويظهر أيضًا للاعبي RICO"
        }
        "A .png skin file (64×64 or 64×32). You can also drop files here." => {
            "ملف مظهر ‎.png‏ (64×64 أو 64×32). يمكنك أيضًا إسقاط الملفات هنا."
        }
        "Drop .png skin files here, add one with the + card, or copy a player's skin." => {
            "أسقط ملفات مظاهر ‎.png هنا، أو أضف مظهرًا من بطاقة +، أو انسخ مظهر لاعب."
        }
        "Right-click a skin to rename, share or delete it." => {
            "انقر بزر الفأرة الأيمن على مظهر لإعادة تسميته أو مشاركته أو حذفه."
        }
        "Changes your real Minecraft skin, seen by everyone" => {
            "يغيّر مظهرك الحقيقي في Minecraft، وسيشاهده الجميع"
        }
        "Look up" => "بحث",
        "Your own…" => "عباءتك الخاصة…",
        "No cape" => "لا توجد عباءة",
        "Your cape" => "عباءتك",
        "Minecraft capes · seen by everyone" => "عباءات Minecraft — يراها الجميع",
        "None" => "لا شيء",
        "Cosmetics are loading." => "جارٍ تحميل العناصر التجميلية.",
        "Nothing here yet." => "لا يوجد شيء هنا بعد.",
        "One per slot. Point at one to try it on; click to wear it, again to take it off." => {
            "عنصر واحد لكل موضع. أشر إلى عنصر لتجربته؛ انقر لارتدائه وانقر مرة أخرى لخلعه."
        }
        "Point at one to try it on. Wearing needs your RICO look, which isn't here yet." => {
            "أشر إلى عنصر لتجربته. يتطلب ارتداؤه تحميل مظهرك في RICO، ولم يكتمل ذلك بعد."
        }
        "Point at one to try it on. Add an account to wear it." => {
            "أشر إلى عنصر لتجربته. أضف حسابًا لارتدائه."
        }
        "Moves it half a pixel at a time, to sit on your skin's eyes." => {
            "يحرّكه بمقدار نصف بكسل في كل مرة ليجلس فوق عيني مظهرك."
        }
        "Up" => "أعلى",
        "Down" => "أسفل",
        "Closer" => "أقرب",
        "Farther" => "أبعد",
        "Reset" => "إعادة ضبط",
        "Skins, capes and cosmetics are worn by an account. Add one on the Accounts tab." => {
            "ترتدي الحسابات المظاهر والعباءات والعناصر التجميلية. أضف حسابًا من تبويب الحسابات."
        }
        "Add an account to wear this" => "أضف حسابًا لارتداء هذا",
        "Can't reach the RICO looks server" => "تعذّر الوصول إلى خادم مظاهر RICO",
        "Nothing can be worn until it answers. Check your connection; the launcher keeps trying." => {
            "لا يمكن ارتداء أي عنصر حتى يستجيب الخادم. تحقق من اتصالك؛ وسيواصل المشغّل المحاولة."
        }
        "RICO looks are unavailable" => "مظاهر RICO غير متاحة",
        "Still signing in to RICO" => "ما زال تسجيل الدخول إلى RICO جاريًا",
        "One moment — your look is loading." => "لحظة واحدة — جارٍ تحميل مظهرك.",
        "Could not read your skin library" => "تعذّرت قراءة مكتبة مظاهرك",
        "Could not read skin" => "تعذّرت قراءة المظهر",
        "Could not add skin" => "تعذّرت إضافة المظهر",
        "Could not delete skin" => "تعذّر حذف المظهر",
        "Could not save skin" => "تعذّر حفظ المظهر",
        "Could not read file" => "تعذّرت قراءة الملف",
        "Already in your library" => "المظهر موجود في مكتبتك بالفعل",
        "Could not read that cape" => "تعذّرت قراءة تلك العباءة",
        "Could not remove that cape" => "تعذّرت إزالة تلك العباءة",
        "Couldn't save your look" => "تعذّر حفظ مظهرك",
        "Minecraft skin change failed" => "فشل تغيير مظهر Minecraft",
        "No OptiFine cape" => "لا توجد عباءة OptiFine",
        "Could not reach OptiFine's cape server" => "تعذّر الوصول إلى خادم عباءات OptiFine",
        "Adventure" => "مغامرات",
        "Fantasy" => "خيال",
        "Gamer" => "لاعبون",
        "Animals" => "حيوانات",
        "PvP" => "قتال لاعب ضد لاعب",
        "Casual" => "ترفيهي",
        "Community" => "المجتمع",
        "Popular" => "الأكثر شعبية",
        "No skins match this collection and search." => {
            "لا توجد مظاهر تطابق هذه المجموعة وعبارة البحث."
        }
        "Add to library" => "إضافة إلى المكتبة",
        "Share to gallery" => "المشاركة في المعرض",
        "Name your skin" => "سمِّ مظهرك",
        "Share" => "مشاركة",
        "Gallery" => "المعرض",
        "Loading friends…" => "جارٍ تحميل الأصدقاء…",
        "Add a friend" => "إضافة صديق",
        "Add friend" => "إضافة صديق",
        "Your code" => "رمزك",
        "Copy your friend code" => "نسخ رمز الصديق",
        "Friend code copied" => "تم نسخ رمز الصديق",
        "Privacy & profile" => "الخصوصية والملف الشخصي",
        "Shown to friends as" => "يظهر للأصدقاء باسم",
        "Friends can see when I'm online" => "يمكن للأصدقاء رؤية حالة اتصالي",
        "…and which server I'm playing on" => "…والخادم الذي ألعب عليه",
        "Friends can invite me" => "يمكن للأصدقاء دعوتي",
        "Show friends all my linked accounts" => "إظهار كل حساباتي المرتبطة للأصدقاء",
        "Unlink" => "إلغاء الربط",
        "Link Discord" => "ربط Discord",
        "Link to this profile" => "ربط بملف التعريف هذا",
        "Copy" => "نسخ",
        "Restore" => "استعادة",
        "Restore with Discord" => "الاستعادة باستخدام Discord",
        "Dismiss" => "تجاهل",
        "Decline" => "رفض",
        "Accept" => "قبول",
        "Cancel request" => "إلغاء الطلب",
        "Remove friend" => "إزالة الصديق",
        "Invite" => "دعوة",
        "Duel" => "مبارزة",
        "Chat" => "محادثة",
        "RICO is an open-source Minecraft launcher for managing game versions, instances, mods, accounts, and friends. It includes the RICO Client and tools for organizing your Minecraft setup." => {
            "RICO هو مشغّل Minecraft مفتوح المصدر لإدارة إصدارات اللعبة والنسخ والإضافات والحسابات والأصدقاء. ويتضمن عميل RICO وأدوات لتنظيم إعدادات Minecraft لديك."
        }
        "Could not get skin" => "تعذّر جلب المظهر",
        "Start the game maximized" => "بدء اللعبة بحجم مكبّر",
        "Storage" => "التخزين",
        "Share & back up" => "المشاركة والنسخ الاحتياطي",
        "A few seconds after the launcher opens, so it starts fast" => {
            "بعد ثوانٍ قليلة من فتح المشغّل، حتى يبدأ بسرعة"
        }
        "After an update, a popup lists what's new and what's fixed" => {
            "بعد التحديث، تعرض نافذة منبثقة الميزات الجديدة والإصلاحات."
        }
        "Aurora" => "الشفق القطبي",
        "Glass" => "زجاجي",
        "Dark" => "داكن",
        "Light" => "فاتح",
        "Your newest screenshot, or the version's title screen" => {
            "أحدث لقطة شاشة لك أو شاشة عنوان الإصدار"
        }
        "Choose a picture of your own" => "اختر صورة من جهازك",
        "Aurora, snowfall and shooting stars. Pauses while you play." => {
            "الشفق القطبي وتساقط الثلوج والنجوم المتحركة. تتوقف أثناء اللعب."
        }
        "Off for now: your graphics driver draws in software (no GPU), where this slows the whole PC down. The glass blur is off too." => {
            "متوقف حاليًا: برنامج تشغيل الرسوميات يعمل بالبرمجيات دون معالج رسومي، وهذا يبطئ الجهاز كله. كما تم إيقاف ضبابية الزجاج."
        }
        "Freeze animated capes and cosmetics" => "تجميد العباءات والعناصر التجميلية المتحركة",
        "Reopens instantly. Quit from the tray icon." => {
            "يُعاد فتحه فورًا. اخرج من خلال أيقونة منطقة الإشعارات."
        }
        "Off: games keep running after the launcher closes." => {
            "عند الإيقاف: تستمر الألعاب في العمل بعد إغلاق المشغّل."
        }
        "Join on a server asks first when no game is running." => {
            "سيُطلب التأكيد قبل الانضمام إلى خادم عند عدم وجود لعبة قيد التشغيل."
        }
        "A tip about one of RICO's features in the sidebar, a new one every 5 minutes." => {
            "نصيحة عن إحدى ميزات RICO في الشريط الجانبي، وتتغير كل 5 دقائق."
        }
        "How Minecraft's menus look in game. Press Right Shift in game for HUD widgets and capes." => {
            "شكل قوائم Minecraft داخل اللعبة. اضغط Shift الأيمن داخل اللعبة لفتح عناصر HUD والعباءات."
        }
        "Play with RICO Client" => "اللعب باستخدام عميل RICO",
        "Gentle launch" => "تشغيل تدريجي",
        "Downloads a few files at a time and starts the game at low priority, so a computer that freezes under load stays usable. Starting takes longer." => {
            "ينزّل بضعة ملفات في كل مرة ويبدأ اللعبة بأولوية منخفضة، ليبقى الجهاز قابلًا للاستخدام تحت الضغط. سيستغرق التشغيل وقتًا أطول."
        }
        "4 GB suits vanilla. Giving Java more than it needs can cause stutter." => {
            "تتناسب 4 جيجابايت مع النسخة الأصلية. منح Java ذاكرة أكبر من حاجته قد يسبب التقطّع."
        }
        "Weekly development builds. They can break worlds; back up first." => {
            "إصدارات تطوير أسبوعية. قد تتسبب في تلف العوالم؛ أنشئ نسخة احتياطية أولًا."
        }
        "The game window fills the screen, with the taskbar and title bar still there." => {
            "تملأ نافذة اللعبة الشاشة مع بقاء شريط المهام وشريط العنوان ظاهرين."
        }
        "Use the gaming graphics card" => "استخدام بطاقة الرسوميات عالية الأداء",
        "Show what you're playing on Discord" => "إظهار ما تلعبه على Discord",
        "Show server in Discord status" => "إظهار الخادم في حالة Discord",
        "Turn on \"Show what you're playing on Discord\" first." => {
            "فعّل أولًا خيار «إظهار ما تلعبه على Discord»."
        }
        "Ask me" => "اسألني",
        "Always send" => "الإرسال دائمًا",
        "Never" => "عدم الإرسال أبدًا",
        "RICO downloads the right Java for each version automatically." => {
            "ينزّل RICO إصدار Java المناسب لكل إصدار تلقائيًا."
        }
        "Updates are managed by your Linux software manager." => {
            "تتم إدارة التحديثات بواسطة مدير برامج Linux لديك."
        }
        "Restart by itself when waiting in the tray" => {
            "إعادة التشغيل تلقائيًا عند الانتظار في منطقة الإشعارات"
        }
        "Update installed - Restart now" => "تم تثبيت التحديث — إعادة التشغيل الآن",
        "Kept the same in every instance of this profile: whatever you changed last is used everywhere, from the next start." => {
            "تبقى هذه الإعدادات موحّدة في كل نسخ ملف التعريف؛ ويُستخدم آخر تغيير في كل مكان عند التشغيل التالي."
        }
        "RICO Client settings (HUD, crosshair, keys, waypoints)" => {
            "إعدادات عميل RICO ‏(HUD ومؤشر التصويب والمفاتيح ونقاط الطريق)"
        }
        "Minecraft settings (keys, video, sound, FOV)" => {
            "إعدادات Minecraft ‏(المفاتيح والفيديو والصوت ومجال الرؤية)"
        }
        "Game settings for new instances" => "إعدادات اللعبة للنسخ الجديدة",
        "Your settings, instances (as mod lists) and HUD setups, as a code, text or file. Accounts and passwords are never included." => {
            "إعداداتك ونسخك (كقوائم إضافات) وتخطيطات HUD على شكل رمز أو نص أو ملف. لا تُضمّن الحسابات وكلمات المرور مطلقًا."
        }
        "Export profile" => "تصدير ملف التعريف",
        "From another launcher" => "من مشغّل آخر",
        "Desktop shortcut" => "اختصار سطح المكتب",
        "Puts an RICO shortcut on your desktop" => "يضع اختصار RICO على سطح المكتب",
        "Desktop shortcut made" => "تم إنشاء اختصار سطح المكتب",
        "Couldn't make the shortcut" => "تعذّر إنشاء الاختصار",
        "Use a custom Java executable" => "استخدام ملف Java تنفيذي مخصّص",
        "File not found; the managed runtime will be used." => {
            "لم يُعثر على الملف؛ سيُستخدم وقت التشغيل المُدار."
        }
        "Where your time goes. It stays on this PC." => {
            "إلى أين يذهب وقتك. تبقى هذه البيانات على هذا الجهاز."
        }
        "Nothing yet" => "لا توجد بيانات بعد",
        "Play for a while and this page fills up: time per instance, per server and per singleplayer world, and how long you spent in menus." => {
            "العب لبعض الوقت وستمتلئ هذه الصفحة: الوقت لكل نسخة وخادم وعالم فردي، والمدة التي قضيتها في القوائم."
        }
        "Total" => "الإجمالي",
        "Game open" => "اللعبة مفتوحة",
        "In a world or server" => "داخل عالم أو خادم",
        "In menus" => "داخل القوائم",
        "Launcher open" => "المشغّل مفتوح",
        "Sessions" => "الجلسات",
        "Hide server and world names" => "إخفاء أسماء الخوادم والعوالم",
        "Server" => "خادم",
        "World" => "عالم",
        "Removed instance" => "نسخة محذوفة",
        "last" => "آخر نشاط",
        "and" => "و",
        "session" => "جلسة",
        "sessions" => "جلسات",
        "{name}…" => "{name}…",
        "Bedrock Edition is played on Windows" => "تُلعَب نسخة Bedrock على Windows",
        "Installing {}" => "جارٍ تثبيت {}",
        "Minecraft {}" => "Minecraft {}",
        "Version {}" => "الإصدار {}",
        "Client" => "العميل",
        "Get Gaming Services" => "الحصول على Gaming Services",
        "Loading the version list…" => "جارٍ تحميل قائمة الإصدارات…",
        "The version list couldn't be loaded: {e}" => "تعذّر تحميل قائمة الإصدارات: {e}",
        "Try again" => "إعادة المحاولة",
        "Older kind" => "نوع أقدم",
        "Show all {}" => "عرض الكل ({})",
        "{} worlds" => "{} عوالم",
        "Import .mcworld" => "استيراد .mcworld",
        "Open the worlds folder" => "فتح مجلد العوالم",
        "Drop .mcworld, .mcpack or .mcaddon files here to import them." => {
            "أفلت ملفات .mcworld أو .mcpack أو .mcaddon هنا لاستيرادها."
        }
        "Couldn't rename it" => "تعذّرت إعادة التسمية",
        "Move to trash" => "نقل إلى سلة المهملات",
        "Moved {} to trash" => "نُقل {} إلى سلة المهملات",
        "Couldn't move the world" => "تعذّر نقل العالم",
        "Export as .mcworld" => "تصدير بصيغة .mcworld",
        "Open the world's folder" => "فتح مجلد العالم",
        "{} packs" => "{} حزم",
        "Import .mcpack / .mcaddon" => "استيراد .mcpack / .mcaddon",
        "Open the packs folder" => "فتح مجلد الحزم",
        "{}S" => "{}S",
        "v{}" => "v{}",
        "Open the pack's folder" => "فتح مجلد الحزمة",
        "Add server" => "إضافة خادم",
        "Servers you add (here or in the game's Servers tab) show up here with who's online." => {
            "تظهر هنا الخوادم التي تضيفها (هنا أو في تبويب الخوادم داخل اللعبة)، مع اللاعبين المتصلين."
        }
        "Couldn't remove it" => "تعذّرت الإزالة",
        "POPULAR SERVERS" => "خوادم شائعة",
        "Added {}" => "أُضيف {}",
        "{address} is already on your list" => "{address} موجود بالفعل في قائمتك",
        "Couldn't add it" => "تعذّرت الإضافة",
        "{} / {} online" => "{} / {} متصلون",
        "Remove from the list" => "إزالة من القائمة",
        "Add" => "إضافة",
        "{} ms" => "{} مللي ثانية",
        "Installed {name}" => "تم تثبيت {name}",
        "Stopped installing {name}" => "توقف تثبيت {name}",
        "Couldn't install {name}" => "تعذّر تثبيت {name}",
        "1 {unit} ago" => "منذ 1 {unit}",
        "{n} {unit}s ago" => "منذ {n} {unit}",
        "(screenshot unavailable)" => "(لقطة الشاشة غير متاحة)",
        "Chat with {name}" => "الدردشة مع {name}",
        "Close chat" => "إغلاق الدردشة",
        "No messages yet. Say hi!" => "لا رسائل بعد. ألقِ التحية!",
        "Send a screenshot" => "إرسال لقطة شاشة",
        "Message {name}" => "مراسلة {name}",
        "No screenshots yet (F2 in game)." => "لا لقطات شاشة بعد (F2 داخل اللعبة).",
        "chat/{id}" => "chat/{id}",
        "Couldn't load the chat" => "تعذّر تحميل الدردشة",
        "Message not sent" => "لم تُرسَل الرسالة",
        "Fancy" => "فاخر",
        "Sign-in failed" => "فشل تسجيل الدخول",
        "Choose a username" => "اختر اسم مستخدم",
        "Go to {} and enter:" => "انتقل إلى {} وأدخل:",
        "Open sign-in page" => "فتح صفحة تسجيل الدخول",
        "Remove {}?" => "إزالة {}؟",
        "You can add it again at any time." => "يمكنك إضافته مجددًا في أي وقت.",
        "Removed {}" => "أُزيل {}",
        "Turn off performance mods?" => "تعطيل تعديلات الأداء؟",
        "Keep them" => "إبقاؤها",
        "Turn off" => "تعطيل",
        "Let the game play as {name}?" => "السماح للعبة باللعب باسم {name}؟",
        "Yes, always" => "نعم، دائمًا",
        "Yes, this time" => "نعم، هذه المرة",
        "Discover servers" => "اكتشاف الخوادم",
        "A new random order every time. Nobody pays for a spot." => {
            "ترتيب عشوائي جديد كل مرة. لا أحد يدفع مقابل الظهور."
        }
        "List your server" => "إدراج خادمك",
        "Surprise me" => "فاجئني",
        "Getting the list…" => "جارٍ جلب القائمة…",
        "The list couldn't be loaded: {e}" => "تعذّر تحميل القائمة: {e}",
        "No servers match." => "لا خوادم مطابقة.",
        "No servers to pick from" => "لا خوادم للاختيار منها",
        "Add to your server list" => "إضافة إلى قائمة خوادمك",
        "It's in {}'s server list." => "موجود في قائمة خوادم {}.",
        "{} is already on your list" => "{} موجود بالفعل في قائمتك",
        "About it" => "نبذة عنه",
        "Tags" => "الوسوم",
        "Submit for review" => "إرسال للمراجعة",
        "Friends couldn't be loaded: {e}" => "تعذّر تحميل الأصدقاء: {e}",
        "مرتبط بـ {name}" => "مرتبط بـ {name}",
        "Linked to {name}" => "مرتبط بـ {name}",
        "Couldn't make a recovery code" => "تعذّر إنشاء رمز استرداد",
        "{} invited you to {what}" => "دعاك {} إلى {what}",
        "يلعب على {s}" => "يلعب على {s}",
        "playing on {s}" => "يلعب على {s}",
        "Invite to {s}" => "دعوة إلى {s}",
        "Start and join {server}" => "تشغيل والانضمام إلى {server}",
        "{} invited you" => "دعاك {}",
        "to {what}" => "إلى {what}",
        "Plus {} more (keys, video, chat…)" => "و{} أخرى (المفاتيح والفيديو والدردشة…)",
        "Couldn't save the defaults" => "تعذّر حفظ الإعدادات الافتراضية",
        "Copy its settings" => "نسخ إعداداته",
        "Copied {n} settings" => "تم نسخ {n} إعدادات",
        "From {}" => "من {}",
        "Apply to it" => "تطبيق عليه",
        "Close that game first" => "أغلق تلك اللعبة أولًا",
        "It saves its own settings when it quits." => "تحفظ إعداداتها عند إغلاقها.",
        "Applied {n} settings" => "تم تطبيق {n} إعدادات",
        "To {}" => "إلى {}",
        "Couldn't apply them" => "تعذّر تطبيقها",
        "Use a code" => "استخدام رمز",
        "game default ({})" => "الافتراضي للعبة ({})",
        "{v}" => "{v}",
        "Search failed: {e}" => "فشل البحث: {e}",
        "by {}" => "بواسطة {}",
        "{} downloads" => "{} تنزيلات",
        "{:.1}K" => "{:.1}K",
        "{:.1}M" => "{:.1}M",
        "{} is ready to play." => "{} جاهز للعب.",
        "{n} problems will stop the game" => "{n} مشكلات ستمنع تشغيل اللعبة",
        "Getting it…" => "جارٍ جلبه…",
        "Could not turn the mod off" => "تعذّر تعطيل التعديل",
        "Vanilla" => "Vanilla",
        "{loader} {}" => "{loader} {}",
        "MINECRAFT VERSION" => "إصدار Minecraft",
        "No versions available for this loader." => "لا إصدارات متاحة لهذا المحمّل.",
        "{} VERSION" => "إصدار {}",
        "{}  (stable)" => "{}  (مستقر)",
        "No loader builds for this version." => "لا بُنى للمحمّل لهذا الإصدار.",
        "Created {}" => "تم إنشاء {}",
        "Could not create instance" => "تعذّر إنشاء نسخة اللعبة",
        "Use a custom amount for this instance" => "استخدام مقدار مخصص لهذه النسخة",
        "{:.1} GB" => "{:.1} GB",
        "The folder (worlds and mods included) is moved to instances/.trash, not deleted." => {
            "يُنقل المجلد (بما فيه العوالم والتعديلات) إلى instances/.trash، ولا يُحذف."
        }
        "Copy the worlds too (can be big)" => "نسخ العوالم أيضًا (قد تكون كبيرة)",
        "Copying…" => "جارٍ النسخ…",
        "Close the game first." => "أغلق اللعبة أولًا.",
        "Made {}" => "تم إنشاء {}",
        "Couldn't duplicate the instance" => "تعذّر إنشاء نسخة مطابقة",
        "Use a specific Java for this instance" => "استخدام Java محددة لهذه النسخة",
        "That file doesn't exist; the managed Java will be used." => {
            "هذا الملف غير موجود؛ ستُستخدم Java المُدارة."
        }
        "RICO downloads the right Java for each Minecraft version." => {
            "ينزّل RICO نسخة Java المناسبة لكل إصدار من Minecraft."
        }
        "The RICO Client is off for every instance" => "عميل RICO معطّل في جميع نسخ اللعبة",
        "Not available for this Minecraft version yet." => "غير متاح لهذا الإصدار من Minecraft بعد.",
        "Ones you've added yourself (or mods that do the same job) are left as they are." => {
            "تُترك التعديلات التي أضفتها بنفسك (أو التي تؤدي الوظيفة نفسها) كما هي."
        }
        "Optional mods that aren't about speed. Each has its own switch." => {
            "تعديلات اختيارية لا تتعلق بالسرعة. لكل منها مفتاح تشغيل مستقل."
        }
        "{name}: {about}" => "{name}: {about}",
        "Couldn't fix it" => "تعذّر الإصلاح",
        "That instance is gone." => "لم تعد نسخة اللعبة تلك موجودة.",
        "Fixed" => "تم الإصلاح",
        "Moved to instances/.trash" => "نُقل إلى instances/.trash",
        "Could not remove instance" => "تعذّرت إزالة نسخة اللعبة",
        "Snowflake" => "ندفة ثلج",
        "Color" => "اللون",
        "Mod loader" => "محمّل التعديلات",
        "Couldn't list the builds: {e}" => "تعذّر عرض البُنى: {e}",
        "Loading builds…" => "جارٍ تحميل البُنى…",
        "{} build" => "بنية {}",
        "Use this build" => "استخدام هذه البنية",
        "Now on {} {choice}" => "يُستخدم الآن {} {choice}",
        "No other loader has a build for Minecraft {game}." => {
            "لا يوفّر أي محمّل آخر بنية لـ Minecraft {game}."
        }
        "Check mods" => "فحص التعديلات",
        "Looking up the mods on Modrinth…" => "جارٍ البحث عن التعديلات على Modrinth…",
        "Couldn't check the mods: {e}" => "تعذّر فحص التعديلات: {e}",
        "{} (needed by another mod)" => "{} (يتطلبه تعديل آخر)",
        "→ {v}" => "→ {v}",
        "Show all {} mods" => "عرض كل التعديلات ({})",
        "Switch to {}" => "التبديل إلى {}",
        "{} mods updated" => "تم تحديث {} تعديلات",
        ", turned off: {}" => "، عُطّل: {}",
        ", couldn't download: {}" => "، تعذّر تنزيل: {}",
        "Now on {}" => "يُستخدم الآن {}",
        "Could not change the loader" => "تعذّر تغيير المحمّل",
        "Loading more…" => "جارٍ تحميل المزيد…",
        "{} {loader} · Minecraft {game}" => "{} {loader} · Minecraft {game}",
        "Minecraft {game}" => "Minecraft {game}",
        "Trash ({removed})" => "سلة المهملات ({removed})",
        "Added {count} mod{}" => "تمت إضافة {count} من التعديلات{}",
        "{} could not be updated." => "تعذّر تحديث {}.",
        "Installed {first}" => "تم تثبيت {first}",
        "Installed {first} + {} dependencies" => "تم تثبيت {first} + {} متطلبات",
        "Import .mrpack…" => "استيراد .mrpack…",
        "Could not load the versions: {e}" => "تعذّر تحميل الإصدارات: {e}",
        "Installed {}" => "تم تثبيت {}",
        "Could not install modpack" => "تعذّر تثبيت حزمة التعديلات",
        "{} mods · {enabled} enabled" => "{} تعديلات · {enabled} مفعّلة",
        "Could not refresh mods: {error}" => "تعذّر تحديث قائمة التعديلات: {error}",
        "{} of {} mods" => "{} من {} تعديلات",
        "Couldn't look for updates: {e}" => "تعذّر البحث عن تحديثات: {e}",
        "Couldn't add {}" => "تعذّرت إضافة {}",
        "{added} mods added" => "تمت إضافة {added} تعديلات",
        "{} · dependency" => "{} · متطلب",
        "None yet: find some below." => "لا شيء بعد: ابحث أدناه.",
        "{:.1} MB" => "{:.1} MB",
        "Couldn't save the group" => "تعذّر حفظ المجموعة",
        "Couldn't install it" => "تعذّر التثبيت",
        "Back to results" => "العودة إلى النتائج",
        "Project preview · Esc to close" => "معاينة المشروع · Esc للإغلاق",
        "This project has no gallery screenshots yet." => {
            "لا توجد لقطات شاشة في معرض هذا المشروع بعد."
        }
        "Supported Minecraft versions" => "إصدارات Minecraft المدعومة",
        "Loaders / platforms" => "المحمّلات / المنصات",
        "Image URL unavailable" => "رابط الصورة غير متاح",
        "Screenshot unavailable" => "لقطة الشاشة غير متاحة",
        "Project" => "المشروع",
        "é {image}![next](https://example.com/next) end" => {
            "é {image}![التالي](https://example.com/next) نهاية"
        }
        "é {placeholder}{placeholder} end" => "é {placeholder}{placeholder} نهاية",
        "Sodium" => "Sodium",
        "Lithium" => "Lithium",
        "FerriteCore" => "FerriteCore",
        "ImmediatelyFast" => "ImmediatelyFast",
        "Entity Culling" => "Entity Culling",
        "ModernFix" => "ModernFix",
        "Iris Shaders" => "Iris Shaders",
        "{} replays" => "{} إعادات",
        "Open replays folder" => "فتح مجلد الإعادات",
        "No replays yet" => "لا إعادات بعد",
        "Gameplay videos ({})" => "فيديوهات اللعب ({})",
        "Show in folder" => "إظهار في المجلد",
        " · {n} clips" => " · {n} مقاطع",
        "Could not move replay" => "تعذّر نقل الإعادة",
        "Watch" => "مشاهدة",
        "{h}:{m:02}:{sec:02}" => "{h}:{m:02}:{sec:02}",
        "{m}:{sec:02}" => "{m}:{sec:02}",
        "Minecraft version" => "إصدار Minecraft",
        "Change version" => "تغيير الإصدار",
        "Now on {game}" => "يُستخدم الآن {game}",
        "Change to {version}" => "التغيير إلى {version}",
        "Now on {version}" => "يُستخدم الآن {version}",
        "Could not change the version" => "تعذّر تغيير الإصدار",
        "Trash" => "سلة المهملات",
        "Removed instances, with their worlds and mods. Bring one back, or delete it for good." => {
            "نسخ اللعبة المُزالة، مع عوالمها وتعديلاتها. استعد إحداها أو احذفها نهائيًا."
        }
        "Nothing here." => "لا شيء هنا.",
        "Yes, delete" => "نعم، احذف",
        "Worlds included. Sure?" => "يشمل العوالم. هل أنت متأكد؟",
        "Delete for good" => "حذف نهائي",
        "Instance restored" => "تمت استعادة نسخة اللعبة",
        "Could not restore it" => "تعذّرت الاستعادة",
        "Could not delete it" => "تعذّر الحذف",
        "Import worlds" => "استيراد عوالم",
        "From a .zip…" => "من ملف .zip…",
        "Open saves folder" => "فتح مجلد saves",
        "No worlds yet" => "لا عوالم بعد",
        "Could not move world" => "تعذّر نقل العالم",
        "Back up to a .zip" => "نسخ احتياطي إلى .zip",
        "Open world folder" => "فتح مجلد العالم",
        "Copies are made; the originals stay where they are." => {
            "تُنشأ نسخ؛ وتبقى الملفات الأصلية في مكانها."
        }
        "Import {n} worlds" => "استيراد {n} عوالم",
        "World operation failed" => "فشلت العملية على العالم",
        "Copied {count} lines" => "تم نسخ {count} أسطر",
        "Send this log to the staff?" => "إرسال هذا السجل إلى فريق الدعم؟",
        "Nothing to send yet" => "لا شيء لإرساله بعد",
        "Couldn't send the log" => "تعذّر إرسال السجل",
        "{marker}{}" => "{marker}{}",
        "Launcher" => "المشغّل",
        "Auto-scroll" => "تمرير تلقائي",
        "game-{}.log" => "game-{}.log",
        "Open log file" => "فتح ملف السجل",
        "Copy shown lines" => "نسخ الأسطر المعروضة",
        "Send this log to the RICO staff" => "إرسال هذا السجل إلى فريق RICO",
        "Move to RICO" => "الانتقال إلى RICO",
        "Copies instances, mods, worlds and settings from other launchers. The originals stay as they are; accounts never come along." => {
            "ينسخ نسخ اللعبة والتعديلات والعوالم والإعدادات من المشغّلات الأخرى. تبقى الملفات الأصلية كما هي؛ ولا تُنقل الحسابات مطلقًا."
        }
        "Looking for other launchers…" => "جارٍ البحث عن مشغّلات أخرى…",
        "HUD layouts and keys" => "تخطيطات HUD والمفاتيح",
        "{}: {why}" => "{}: {why}",
        "Nothing found. If your launcher keeps instances somewhere else, choose that folder." => {
            "لم يُعثر على شيء. إذا كان مشغّلك يحفظ نسخ اللعبة في مكان آخر، فاختر ذلك المجلد."
        }
        "Choose folder…" => "اختيار مجلد…",
        "{:.1} GB to copy" => "{:.1} GB للنسخ",
        "Bring over {count}" => "نقل {count}",
        "{failed} couldn't come over" => "تعذّر نقل {failed}",
        "See the details in the window." => "راجع التفاصيل في النافذة.",
        "{} · {loader}" => "{} · {loader}",
        "{} ({n}, {mb:.0} MB)" => "{} ({n}, {mb:.0} MB)",
        "{widgets_on} widgets on" => "{widgets_on} عناصر مفعّلة",
        "{} of {} done" => "اكتمل {} من {}",
        "Use a proxy" => "استخدام وكيل",
        "Test proxy" => "اختبار الوكيل",
        "Undo" => "تراجع",
        "The proxy didn't work: {e}" => "لم يعمل الوكيل: {e}",
        "Could not save the proxy" => "تعذّر حفظ الوكيل",
        "The proxy can't be used" => "لا يمكن استخدام الوكيل",
        "{} instance" => "نسخة {}",
        "Could not load versions: {e}" => "تعذّر تحميل الإصدارات: {e}",
        "Could not load versions. Check your internet connection and try again." => {
            "تعذّر تحميل الإصدارات. تحقق من اتصالك بالإنترنت وحاول مرة أخرى."
        }
        "{date} · snapshot" => "{date} · إصدار تجريبي",
        "{date} · old" => "{date} · قديم",
        "Installed ({installed})" => "المثبّتة ({installed})",
        "Favorites ({favorites})" => "المفضلة ({favorites})",
        "{p}%" => "{p}%",
        "{}/{} files" => "{}/{} ملفات",
        "{}/s" => "{}/ث",
        "{} left" => "متبقٍ {}",
        "{}…  ·  {} s" => "{}…  ·  {} ث",
        "Play {}" => "تشغيل {}",
        "{} min ago" => "منذ {} دقيقة",
        "{} h ago" => "منذ {} ساعة",
        "{} days ago" => "منذ {} أيام",
        "New profile" => "ملف شخصي جديد",
        "A profile has its own accounts, settings, instances and worlds." => {
            "لكل ملف شخصي حساباته وإعداداته ونسخ لعبته وعوالمه."
        }
        "Create and switch" => "إنشاء والتبديل",
        "Could not create profile" => "تعذّر إنشاء الملف الشخصي",
        "Profiles" => "الملفات الشخصية",
        "Could not rename" => "تعذّرت إعادة التسمية",
        "Rename" => "إعادة تسمية",
        "Switch" => "تبديل",
        "Remove this profile? Its folder (worlds included) is moved to profiles/.trash, not deleted." => {
            "إزالة هذا الملف الشخصي؟ سيُنقل مجلده (بما فيه العوالم) إلى profiles/.trash، ولن يُحذف."
        }
        "Keep" => "إبقاء",
        "Could not remove profile" => "تعذّرت إزالة الملف الشخصي",
        "Could not save profiles" => "تعذّر حفظ الملفات الشخصية",
        "Quick tip: {text}" => "نصيحة سريعة: {text}",
        "Looking for screenshots…" => "جارٍ البحث عن لقطات الشاشة…",
        "No screenshots yet. Press F2 in game to take one." => {
            "لا لقطات شاشة بعد. اضغط F2 داخل اللعبة لالتقاط واحدة."
        }
        "The file can't be read as a picture ({e})." => "لا يمكن قراءة الملف كصورة ({e}).",
        "Opening the picture…" => "جارٍ فتح الصورة…",
        "Couldn't show this screenshot" => "تعذّر عرض لقطة الشاشة هذه",
        "\\\"Open\\\" below may still work." => "قد يظل زر \\\"فتح\\\" أدناه يعمل.",
        "Delete (moves it to .trash)" => "حذف (ينقلها إلى .trash)",
        "Copy image" => "نسخ الصورة",
        "Copied the screenshot" => "تم نسخ لقطة الشاشة",
        "Couldn't copy it" => "تعذّر النسخ",
        "Sending to {name}…" => "جارٍ الإرسال إلى {name}…",
        "Your server list couldn't be read: {e}" => "تعذّرت قراءة قائمة خوادمك: {e}",
        "Couldn't move the server" => "تعذّر نقل الخادم",
        "Add to list" => "إضافة إلى القائمة",
        "Not a server address" => "ليس عنوان خادم",
        "Pinging…" => "جارٍ اختبار الاتصال…",
        "No answer: {e}" => "لا استجابة: {e}",
        "Added {typed}" => "أُضيف {typed}",
        "{typed} is already on your list" => "{typed} موجود بالفعل في قائمتك",
        "Start {} and join" => "تشغيل {} والانضمام",
        "{ms} ms" => "{ms} مللي ثانية",
        "{} +{} more" => "{} +{} أخرى",
        "Choose another picture. Now: {}" => "اختر صورة أخرى. الحالية: {}",
        "{w} × {h}" => "{w} × {h}",
        "{f} isn't on Modrinth, so it's left out; send it separately." => {
            "{f} غير موجود على Modrinth، لذا استُبعد؛ أرسله منفصلًا."
        }
        "Codes are kept on the RICO server; text and files work anywhere." => {
            "تُحفظ الرموز على خادم RICO؛ ويعمل النص والملفات في أي مكان."
        }
        "Copied as text" => "نُسخ كنص",
        "arctic-{}.json" => "arctic-{}.json",
        "Copied. Friends paste it into Import (Instances tab) or run `arctic import`." => {
            "تم النسخ. يمكن للأصدقاء لصقه في «استيراد» (تبويب نسخ اللعبة) أو تشغيل `arctic import`."
        }
        "Paste a code (like abcd-efgh) or share text, or open a .json file." => {
            "الصق رمزًا (مثل abcd-efgh) أو نص مشاركة، أو افتح ملف .json."
        }
        " and {more} more" => " و{more} أخرى",
        "It becomes a new instance; the exact same mod versions are downloaded from Modrinth." => {
            "تُنشأ نسخة لعبة جديدة؛ وتُنزّل إصدارات التعديلات نفسها تمامًا من Modrinth."
        }
        "These become your default game settings: every new instance starts with them. Your instances keep theirs." => {
            "تصبح هذه إعدادات لعبتك الافتراضية: تبدأ بها كل نسخة جديدة. وتحتفظ نسخك الحالية بإعداداتها."
        }
        "Launcher settings ({})" => "إعدادات المشغّل ({})",
        "Vanilla instance's setup" => "إعدادات نسخة Vanilla",
        "Default game settings ({})" => "إعدادات اللعبة الافتراضية ({})",
        "{} ({} mods)" => "{} ({} تعديلات)",
        "Ticked launcher settings replace yours. Ticked instances are added; yours with the same names stay as they are." => {
            "تحل إعدادات المشغّل المحددة محل إعداداتك. تُضاف نسخ اللعبة المحددة؛ وتبقى نسخك التي تحمل الأسماء نفسها كما هي."
        }
        "Saved to {}" => "حُفظ في {}",
        "\\n…and {} more" => "\\n…و{} أخرى",
        "Some things were left out" => "استُبعدت بعض العناصر",
        "…and {} more" => "…و{} أخرى",
        "RICO {} is available." => "يتوفر RICO {}.",
        "Update" => "تحديث",
        "Update failed: {error}" => "فشل التحديث: {error}",
        "{e}. Please reopen the launcher." => "{e}. يُرجى إعادة فتح المشغّل.",
        "{} arms" => "أذرع {}",
        "Your cape {}" => "رداؤك {}",
        "Fit {name}" => "ضبط مقاس {name}",
        "{name} on the other shoulder" => "{name} على الكتف الأخرى",
        "{key}{}" => "{key}{}",
        "cape-decode-{i}" => "cape-decode-{i}",
        "OptiFine has no cape for {name}." => "لا يوجد رداء لـ {name} على OptiFine.",
        "{key}#{i}" => "{key}#{i}",
        "{flat}," => "{flat},",
        "{flat}}}" => "{flat}}}",
        "RICO looks are unavailable: {e}" => "مظاهر RICO غير متاحة: {e}",
        "{} (Minecraft)" => "{} (Minecraft)",
        "Could not open folder" => "تعذّر فتح المجلد",
        "Set by the ARCTIC_DATA_DIR environment variable." => "يحدده متغير البيئة ARCTIC_DATA_DIR.",
        "Nothing changed: RICO still uses {}." => "لم يتغير شيء: لا يزال RICO يستخدم {}.",
        "This instance's worlds, mods, resource packs, screenshots and options:" => {
            "عوالم هذه النسخة وتعديلاتها وحزم مواردها ولقطات شاشتها وخياراتها:"
        }
        "Change…" => "تغيير…",
        "Use the instance's own folder" => "استخدام مجلد النسخة الخاص",
        "Close this instance to change its folder." => "أغلق هذه النسخة لتغيير مجلدها.",
        "Pick a folder on any drive, or an existing .minecraft-style folder to use as it is." => {
            "اختر مجلدًا على أي قرص، أو مجلدًا موجودًا ببنية .minecraft لاستخدامه كما هو."
        }
        "Moving the instance's files…" => "جارٍ نقل ملفات النسخة…",
        "Use the instance's own folder?" => "استخدام مجلد النسخة الخاص؟",
        "Use this game folder?" => "استخدام مجلد اللعبة هذا؟",
        "Move this instance's worlds, mods and settings from {} there?" => {
            "نقل عوالم هذه النسخة وتعديلاتها وإعداداتها من {} إلى هناك؟"
        }
        "{e}. Your text is still here." => "{e}. لا يزال نصك هنا.",
        "launcher log" => "سجل المشغّل",
        "game log" => "سجل اللعبة",
        "(couldn't read {})" => "(تعذّرت قراءة {})",
        "==== {title} ====\\n" => "==== {title} ====\\n",
        "Sharing \\\"{}\\\"" => "جارٍ مشاركة \\\"{}\\\"",
        "{n} friends connected" => "{n} أصدقاء متصلون",
        "Connected to \\\"{motd}\\\"" => "متصل بـ \\\"{motd}\\\"",
        "Hosting a world · {guests} friends" => "استضافة عالم · {guests} أصدقاء",
        "Could not start networking: {e}" => "تعذّر بدء الاتصال الشبكي: {e}",
        "Could not create your invite key: {e}" => "تعذّر إنشاء مفتاح دعوتك: {e}",
        "A friend joined" => "انضم صديق",
        "RICO {}" => "RICO {}",
        "Restarting in {seconds_left} s…" => "إعادة التشغيل خلال {seconds_left} ث…",
        "Updating in {seconds_left} s…" => "التحديث خلال {seconds_left} ث…",
        "Installed. The launcher restarts when you close Minecraft." => {
            "تم التثبيت. سيُعاد تشغيل المشغّل عند إغلاق Minecraft."
        }
        "Installed. {countdown}" => "تم التثبيت. {countdown}",
        "The update failed: {error}" => "فشل التحديث: {error}",
        "Updated to RICO {}" => "تم التحديث إلى RICO {}",
        "{db:.0} dB" => "{db:.0} dB",
        "{} player(s) muted" => "تم كتم {} من اللاعبين",
        "PLAY" => "تشغيل",
        "PLAYING" => "قيد التشغيل",
        "SERVERS" => "الخوادم",
        "Quick tip" => "نصيحة سريعة",
        "Profiles keep accounts and instances apart (top left)." => {
            "تفصل الملفات الشخصية بين الحسابات والنسخ (أعلى اليسار)."
        }
        "Mods that would stop the game get fixed before it starts." => {
            "تُصلح الإضافات التي قد توقف اللعبة قبل بدء التشغيل."
        }
        "Drag servers to put your favorites first." => "اسحب الخوادم لوضع المفضلة أولًا.",
        "Discover servers finds new places to play." => "يكتشف الخوادم أماكن جديدة للعب.",
        "Right Shift in game moves or hides any HUD widget." => {
            "ينقل مفتاح Shift الأيمن أي عنصر من واجهة اللعب أو يخفيه."
        }
        "Controllers work in game on every version." => {
            "تعمل أيدي التحكم داخل اللعبة مع جميع الإصدارات."
        }
        "Tier Tags show PvP tiers by names (Right Shift in game)." => {
            "تعرض وسوم المستويات تصنيفات PvP بالأسماء (Shift الأيمن داخل اللعبة)."
        }
        "Capes, wings, pets and emotes are free." => {
            "الأردية والأجنحة والحيوانات الأليفة والتعبيرات مجانية."
        }
        "Cosmetics → Skin gallery: free skins to try on." => {
            "المظهر ← معرض المظاهر: جرب مظاهر مجانية."
        }
        "Glasses off your eyes? Nudge face items in Cosmetics." => {
            "هل النظارات بعيدة عن عينيك؟ حرّك عناصر الوجه من قسم المظهر."
        }
        "Shoulder pets can sit on either shoulder." => "يمكن للحيوانات الأليفة الجلوس على أي كتف.",
        "Play together without a server: share an invite code." => {
            "العبوا معًا دون خادم: شارك رمز دعوة."
        }
        "Friends see where you play and join in one click." => {
            "يرى الأصدقاء مكان لعبك وينضمون بنقرة واحدة."
        }
        "F9 saves the last 30 seconds; F10 starts or stops a recording." => {
            "يحفظ F9 آخر 30 ثانية؛ ويبدأ F10 التسجيل أو يوقفه."
        }
        "All your screenshots, every instance, one place." => {
            "كل لقطات الشاشة لكل النسخ في مكان واحد."
        }
        "Modrinth modpacks install as their own instance." => "تُثبّت حزم Modrinth كنسخ مستقلة.",
        "An instance can keep its worlds on any drive." => "يمكن للنسخة حفظ عوالمها على أي قرص.",
        "Shaders: turn on Iris in an instance's settings." => {
            "التظليل: فعّل Iris من إعدادات النسخة."
        }
        "Short on space? Move RICO in Settings → Storage." => {
            "هل المساحة قليلة؟ انقل RICO من الإعدادات ← التخزين."
        }
        "Bring instances and HUDs from other launchers." => {
            "استورد النسخ وواجهات اللعب من مشغلات أخرى."
        }
        "Share your setup or a HUD as a code." => "شارك إعداداتك أو واجهة اللعب كرمز.",
        "Try the Glass theme in Settings → Appearance." => "جرّب سمة الزجاج من الإعدادات ← المظهر.",
        "One switch adds Sodium and friends for more FPS." => {
            "يضيف مفتاح واحد Sodium ورفاقه لمعدل إطارات أعلى."
        }
        "Voice chat works in shared worlds and SVC servers." => {
            "تعمل المحادثة الصوتية في العوالم المشتركة وخوادم SVC."
        }
        "Animated capes too busy? Freeze them in Settings." => {
            "هل الأردية المتحركة مزعجة؟ أوقف حركتها من الإعدادات."
        }
        "Auto Reconnect brings you back when a server drops you." => {
            "يعيد الاتصال التلقائي ربطك عند انقطاع الخادم."
        }
        "Waypoints and a minimap come with the RICO Client." => {
            "يأتي عميل RICO مع نقاط طريق وخريطة مصغرة."
        }
        "The arctic command does it all too: arctic --help." => {
            "ينفذ أمر arctic كل ذلك أيضًا: arctic --help."
        }
        "Servers you add in Minecraft (or from Discover) show up here with who's online." => {
            "تظهر هنا الخوادم التي تضيفها في Minecraft (أو من الاكتشاف) مع المتصلين."
        }
        "Interface size" => "حجم الواجهة",
        "Play another copy (switch account first for a second player)" => {
            "تشغيل نسخة أخرى (بدّل الحساب أولًا للاعب ثانٍ)"
        }
        _ => english,
    };
    visual_text(translated)
}

/// Owned translation for a dynamic English message. This currently covers the
/// most common shell messages and safely falls back for server/mod names.
pub fn text_owned(language: Language, english: impl Into<String>) -> String {
    let english = english.into();
    text(language, &english).to_owned()
}

/// Apply the common translation to a static label without allocating.
#[macro_export]
macro_rules! tr {
    ($language:expr, $text:literal) => {
        $crate::i18n::text($language, $text)
    };
}
