#![allow(clippy::upper_case_acronyms)]

use std::str::FromStr;

use sabi_commands::CommandParser;
use sabi_messenger::{implement_message, Message, MessageFromString};

use super::state::*;

// Please refer to
// https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum Key {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    /** The user agent wasn't able to map the event's virtual keycode to a specific key value. This can happen due to hardware or software constraints, or because of constraints around the platform on which the user agent is running. */
    Unidentified,
    /** The Alt (Alternative) key. */
    Alt,
    /** The AltGr or AltGraph (Alternate Graphics) key. Enables the ISO Level 3 shift modifier (where Shift is the level 2 modifier). */
    AltGraph,
    /** The Caps Lock key. Toggles the capital character lock on and off for subsequent input. */
    CapsLock,
    /** The Control, Ctrl, or Ctl key. Allows typing control characters. */
    Control,
    /** The Fn (Function modifier) key. Used to allow generating function key (F1-F15, for instance) characters on keyboards without a dedicated function key area. Often handled in hardware so that events aren't generated for this key. */
    Fn,
    /** The FnLock or F-Lock (Function Lock) key.Toggles the function key mode described by "Fn" on and off. Often handled in hardware so that events aren't generated for this key. */
    FnLock,
    /** The Hyper key. */
    Hyper,
    /** The Meta key. Allows issuing special command inputs. This is the Windows logo key, or the Command or ⌘ key on Mac keyboards. */
    Meta,
    /** The NumLock (Number Lock) key. Toggles the numeric keypad between number entry some other mode (often directional arrows). */
    NumLock,
    /** The Scroll Lock key. Toggles beteen scrolling and cursor movement modes. */
    ScrollLock,
    /** The Shift key. Modifies keystrokes to allow typing upper (or other) case letters, and to support typing punctuation and other special characters. */
    Shift,
    /** The Super key. */
    Super,
    /** The Symbol modifier key (found on certain virtual keyboards). */
    Symbol,
    /** The Symbol Lock key. */
    SymbolLock,
    /** The Enter or ↵ key (sometimes labeled Return). */
    Enter,
    /** The Horizontal Tab key, Tab. */
    Tab,
    /** The Space key, ' ' */
    Space,
    /** The down arrow key. */
    ArrowDown,
    /** The left arrow key. */
    ArrowLeft,
    /** The right arrow key. */
    ArrowRight,
    /** The up arrow key. */
    ArrowUp,
    /** The End key. Moves to the end of content. */
    End,
    /** The Home key. Moves to the start of content. */
    Home,
    /** The Page Down (or PgDn) key. Scrolls down or displays the next page of content. */
    PageDown,
    /** The Page Up (or PgUp) key. Scrolls up or displays the previous page of content. */
    PageUp,
    /** The Backspace key. This key is labeled Delete on Mac keyboards. */
    Backspace,
    /** The Clear key. Removes the currently selected input. */
    Clear,
    /** The Copy key (on certain extended keyboards). */
    Copy,
    /** The Cursor Select key, CrSel. */
    CrSel,
    /** The Cut key (on certain extended keyboards). */
    Cut,
    /** The Delete key, Del. */
    Delete,
    /** Erase to End of Field. Deletes all characters from the current cursor position to the end of the current field. */
    EraseEof,
    /** The ExSel (Extend Selection) key. */
    ExSel,
    /** The Insert key, Ins. Toggles  between inserting and overwriting text. */
    Insert,
    /** Paste from the clipboard. */
    Paste,
    /** Redo the last action. */
    Redo,
    /** Undo the last action. */
    Undo,
    /** The Accept, Commit, or OK key or button. Accepts the currently selected option or input method sequence conversion. */
    Accept,
    /** The Again key. Redoes or repeats a previous action. */
    Again,
    /** The Cancel key. */
    Cancel,
    /** Shows the context menu. Typically found between the Windows (or OS) key and the Control key on the right side of the keyboard. */
    ContextMenu,
    /** The Esc (Escape) key. Typically used as an exit, cancel, or "escape this operation" button. Historically, the Escape character was used to signal the start of a special control sequence of characters called an "escape sequence." */
    Escape,
    /** The Execute key. */
    Execute,
    /** The Find key. Opens an interface (typically a dialog box) for performing a find/search operation. */
    Find,
    /** The Finish key. */
    Finish,
    /** The Help key. Opens or toggles the display of help information. */
    Help,
    /**
     * The Pause key. Pauses the current application or state, if applicable.
     * This shouldn't be confused with the "MediaPause" key value, which is used for media controllers, rather than to control applications and processes.
     */
    Pause,
    /**
     * The Play key. Resumes a previously paused application, if applicable.
     * This shouldn't be confused with the "MediaPlay" key value, which is used for media controllers, rather than to control applications and processes.
     */
    Play,
    /** The Props (Properties) key. */
    Props,
    /** The Select key. */
    Select,
    /** The ZoomIn key. */
    ZoomIn,
    /** The ZoomOut key. */
    ZoomOut,
    /** The Brightness Down key. Typically used to reduce the brightness of the display. */
    BrightnessDown,
    /** The Brightness Up key. Typically increases the brightness of the display. */
    BrightnessUp,
    /** The Eject key. Ejects removable media (or toggles an optical storage device tray open and closed). */
    Eject,
    /** The LogOff key. */
    LogOff,
    /**
     * The Power button or key, to toggle power on and off.
     * Not all systems pass this key through to to the user agent.
     */
    Power,
    /** The PowerOff or PowerDown key. Shuts off the system. */
    PowerOff,
    /** The PrintScreen or PrtScr key. Sometimes SnapShot. Captures the screen and prints it or saves it to disk. */
    PrintScreen,
    /** The Hibernate key. This saves the state of the computer to disk and then shuts down; the computer can be returned to its previous state by restoring the saved state information. */
    Hibernate,
    /** The Standby key; also known as Suspend or Sleep. This turns off the display and puts the computer in a low power consumption mode, without completely powering off. */
    Standby,
    /** The WakeUp key; used to wake the computer from the hibernation or standby modes. */
    WakeUp,
    /** The All Candidates key, which starts multi-candidate mode, in which multiple candidates are displayed for the ongoing input. */
    AllCandidates,
    /** The Alphanumeric key. */
    Alphanumeric,
    /** The Code Input key, which enables code input mode, which lets the user enter characters by typing their code points (their Unicode character numbers, typically). */
    CodeInput,
    /** The Compose key. */
    Compose,
    /** The Convert key, which instructs the IME to convert the current input method sequence into the resulting character. */
    Convert,
    /** A dead "combining" key; that is, a key which is used in tandem with other keys to generate accented and other modified characters. If pressed by itself, it doesn't generate a character. If you wish to identify which specific dead key was pressed (in cases where more than one exists), you can do so by examining the KeyboardEvent's associated compositionupdate event's  data property. */
    Dead,
    /** The Final (Final Mode) key is used on some Asian keyboards to enter final mode when using IMEs. */
    FinalMode,
    /** Switches to the first character group on an ISO/IEC 9995 keyboard. Each key may have multiple groups of characters, each in its own column. Pressing this key instructs the device to interpret keypresses as coming from the first column on subsequent keystrokes. */
    GroupFirst,
    /** Switches to the last character group on an ISO/IEC 9995 keyboard. */
    GroupLast,
    /** Switches to the next character group on an ISO/IEC 9995 keyboard. */
    GroupNext,
    /** Switches to the previous character group on an ISO/IEC 9995 keyboard. */
    GroupPrevious,
    /** The Mode Change key. Toggles or cycles among input modes of IMEs. */
    ModeChange,
    /** The Next Candidate function key. Selects the next possible match for the ongoing input. */
    NextCandidate,
    /** The NonConvert ("Don't convert") key. This accepts the current input method sequence without running conversion when using an IME. */
    NonConvert,
    /** The Previous Candidate key. Selects the previous possible match for the ongoing input. */
    PreviousCandidate,
    /** The Process key. Instructs the IME to process the conversion. */
    Process,
    /** The Single Candidate key. Enables single candidate mode (as opposed to multi-candidate mode); in this mode, only one candidate is displayed at a time. */
    SingleCandidate,
    /** The Hangul (Korean character set) mode key, which toggles between Hangul and English entry modes. */
    HangulMode,
    /** Selects the Hanja mode, for converting Hangul characters to the more specific Hanja characters. */
    HanjaMode,
    /** Selects the Junja mode, in which Korean is represented using single-byte Latin characters. */
    JunjaMode,
    /** The Eisu key. This key's purpose is defined by the IME, but may be used to close the IME. */
    Eisu,
    /** The Hankaku (half-width characters) key. */
    Hankaku,
    /** The Hiragana key; selects Kana characters mode. */
    Hiragana,
    /** Toggles between the Hiragana and Katakana writing systems. */
    HiraganaKatakana,
    /** The Kana Mode (Kana Lock) key. */
    KanaMode,
    /** The Kanji Mode key. Enables entering Japanese text using the ideographic characters of Chinese origin. */
    KanjiMode,
    /** The Katakana key. */
    Katakana,
    /** The Romaji key; selects the Roman character set. */
    Romaji,
    /** The Zenkaku (full width) characters key. */
    Zenkaku,
    /** The Zenkaku/Hankaku (full width/half width) toggle key. */
    ZenkakuHanaku,
    /** The first general-purpose function key, F1. */
    F1,
    /** The F2 key. */
    F2,
    /** The F3 key. */
    F3,
    /** The F4 key. */
    F4,
    /** The F5 key. */
    F5,
    /** The F6 key. */
    F6,
    /** The F7 key. */
    F7,
    /** The F8 key. */
    F8,
    /** The F9 key. */
    F9,
    /** The F10 key. */
    F10,
    /** The F11 key. */
    F11,
    /** The F12 key. */
    F12,
    /** The F13 key. */
    F13,
    /** The F14 key. */
    F14,
    /** The F15 key. */
    F15,
    /** The F16 key. */
    F16,
    /** The F17 key. */
    F17,
    /** The F18 key. */
    F18,
    /** The F19 key. */
    F19,
    /** The F20 key. */
    F20,
    /** The first general-purpose virtual function key. */
    Soft1,
    /** The second general-purpose virtual function key. */
    Soft2,
    /** The third general-purpose virtual function key. */
    Soft3,
    /** The fourth general-purpose virtual function key. */
    Soft4,
    /** Presents a list of recently-used applications which lets the user change apps quickly. */
    AppSwitch,
    /** The Call key; dials the number which has been entered. */
    Call,
    /** The Camera key; activates the camera. */
    Camera,
    /** The Focus key; focuses the camera. */
    CameraFocus,
    /** The End Call or Hang Up button. */
    EndCall,
    /** The Back button. */
    GoBack,
    /** The Home button, which takes the user to the phone's main screen (usually an application launcher). */
    GoHome,
    /** The Headset Hook key. This is typically actually a button on the headset which is used to hang up calls and play or pause media. */
    HeadsetHook,
    /** The Redial button, which redials the last-called number. */
    LastNumberRedial,
    /** The Notification key. */
    Notification,
    /** A button which cycles among the notification modes: silent, vibrate, ring, and so forth. */
    MannerMode,
    /** The Voice Dial key. Initiates voice dialing. */
    VoiceDial,
    /** Switches to the previous channel. */
    ChannelDown,
    /** Switches to the next channel. */
    ChannelUp,
    /** Starts, continues, or increases the speed of fast forwarding the media. */
    MediaFastForward,
    /** Pauses the currently playing media. Some older applications use simply "Pause" but this is not correct. */
    MediaPause,
    /** Starts or continues playing media at normal speed, if not already doing so. Has no effect otherwise. */
    MediaPlay,
    /** Toggles between playing and pausing the current media. */
    MediaPlayPause,
    /** Starts or resumes recording media. */
    MediaRecord,
    /** Starts, continues, or increases the speed of rewinding the media. */
    MediaRewind,
    /** Stops the current media activity (such as playing, recording, pausing, forwarding, or rewinding). Has no effect if the media is currently stopped already. */
    MediaStop,
    /** Seeks to the next media or program track. */
    MediaTrackNext,
    /** Seeks to the previous media or program track. */
    MediaTrackPrevious,
    /** Adjusts audio balance toward the left. */
    AudioBalanceLeft,
    /** Adjusts audio balance twoard the right. */
    AudioBalanceRight,
    /** Decreases the amount of bass. */
    AudioBassDown,
    /** Reduces bass boosting or cycles downward through bass boost modes or states. */
    AudioBassBoostDown,
    /** Toggles bass boosting on and off. */
    AudioBassBoostToggle,
    /** Increases the amoung of bass boosting, or cycles upward through a set of bass boost modes or states. */
    AudioBassBoostUp,
    /** Increases the amount of bass. */
    AudioBassUp,
    /** Adjusts the audio fader toward the front. */
    AudioFaderFront,
    /** Adjustts the audio fader toward the rear. */
    AudioFaderRear,
    /** Selects the next available surround sound mode. */
    AudioSurroundModeNext,
    /** Decreases the amount of treble. */
    AudioTrebleDown,
    /** Increases the amount of treble. */
    AudioTrebleUp,
    /** Decreases the audio volume. */
    AudioVolumeDown,
    /** Mutes the audio. */
    AudioVolumeMute,
    /** Increases the audio volume. */
    AudioVolumeUp,
    /** Toggles the microphone on and off. */
    MicrophoneToggle,
    /** Decreases the microphone's input volume. */
    MicrophoneVolumeDown,
    /** Mutes the microphone input. */
    MicrophoneVolumeMute,
    /** Increases the microphone's input volume. */
    MicrophoneVolumeUp,
    /** Switches into TV viewing mode. */
    TV,
    /** Toggles 3D TV mode on and off. */
    TV3DMode,
    /** Toggles between antenna and cable inputs. */
    TVAntennaCable,
    /** Toggles audio description mode on and off. */
    TVAudioDescription,
    /** Decreases trhe audio description's mixing volume; reduces the volume of the audio descriptions relative to the program sound. */
    TVAudioDescriptionMixDown,
    /** Increases the audio description's mixing volume; increases the volume of the audio descriptions relative to the program sound. */
    TVAudioDescriptionMixUp,
    /** Displays or hides the media contents available for playback (this may be a channel guide showing the currently airing programs, or a list of media files to play). */
    TVContentsMenu,
    /** Displays or hides the TV's data service menu. */
    TVDataService,
    /** Cycles the input mode on an external TV. */
    TVInput,
    /** Switches to the input "Component 1." */
    TVInputComponent1,
    /** Switches to the input "Component 2." */
    TVInputComponent2,
    /** Switches to the input "Composite 1." */
    TVInputComposite1,
    /** Switches to the input "Composite 2." */
    TVInputComposite2,
    /** Switches to the input "HDMI 1." */
    TVInputHDMI1,
    /** Switches to the input "HDMI 2." */
    TVInputHDMI2,
    /** Switches to the input "HDMI 3." */
    TVInputHDMI3,
    /** Switches to the input "HDMI 4." */
    TVInputHDMI4,
    /** Switches to the input "VGA 1." */
    TVInputVGA1,
    /** The Media Context menu key. */
    TVMediaContext,
    /** Toggle the TV's network connection on and off. */
    TVNetwork,
    /** Put the TV into number entry mode. */
    TVNumberEntry,
    /** The device's power button. */
    TVPower,
    /** Radio button. */
    TVRadioService,
    /** Satellite button. */
    TVSatellite,
    /** Broadcast Satellite button. */
    TVSatelliteBS,
    /** Communication Satellite button. */
    TVSatelliteCS,
    /** Toggles among available satellites. */
    TVSatelliteToggle,
    /** Selects analog terrestrial television service (analog cable or antenna reception). */
    TVTerrestrialAnalog,
    /** Selects digital terrestrial television service (digital cable or antenna receiption). */
    TVTerrestrialDigital,
    /** Timer programming button. */
    TVTimer,
    /** Changes the input mode on an external audio/video receiver (AVR) unit. */
    AVRInput,
    /** Toggles the power on an external AVR unit. */
    AVRPower,
    /** General-purpose media function key, color-coded red; this has index 0 among the colored keys. */
    ColorF0Red,
    /** General-purpose media funciton key, color-coded green; this has index 1 among the colored keys. */
    ColorF1Green,
    /** General-purpose media funciton key, color-coded yellow; this has index 2 among the colored keys. */
    ColorF2Yellow,
    /** General-purpose media funciton key, color-coded blue; this has index 3 among the colored keys. */
    ColorF3Blue,
    /** General-purpose media funciton key, color-coded grey; this has index 4 among the colored keys. */
    ColorF4Grey,
    /** General-purpose media funciton key, color-coded brown; this has index 5 among the colored keys. */
    ColorF5Brown,
    /** Toggles closed captioning on and off. */
    ClosedCaptionToggle,
    /** Adjusts the brightness of the device by toggling between two brightness levels or by cycling among multiple brightness levels. */
    Dimmer,
    /** Cycles among video sources. */
    DisplaySwap,
    /** Switches the input source to the Digital Video Recorder (DVR). */
    DVR,
    /** The Exit button, which exits the curreent application or menu. */
    Exit,
    /** Clears the program or content stored in the first favorites list slot. */
    FavoriteClear0,
    /** Clears the program or content stored in the second favorites list slot. */
    FavoriteClear1,
    /** Clears the program or content stored in the third favorites list slot. */
    FavoriteClear2,
    /** Clears the program or content stored in the fourth favorites list slot. */
    FavoriteClear3,
    /** Selects (recalls) the program or content stored in the first favorites list slot. */
    FavoriteRecall0,
    /** Selects (recalls) the program or content stored in the second favorites list slot. */
    FavoriteRecall1,
    /** Selects (recalls) the program or content stored in the third favorites list slot. */
    FavoriteRecall2,
    /** Selects (recalls) the program or content stored in the fourth favorites list slot. */
    FavoriteRecall3,
    /** Stores the current program or content into the first favorites list slot. */
    FavoriteStore0,
    /** Stores the current program or content into the second favorites list slot. */
    FavoriteStore1,
    /** Stores the current program or content into the third favorites list slot. */
    FavoriteStore2,
    /** Stores the current program or content into the fourth favorites list slot. */
    FavoriteStore3,
    /** Toggles the display of the program or content guide. */
    Guide,
    /** If the guide is currently displayed, this button tells the guide to display the next day's content. */
    GuideNextDay,
    /** If the guide is currently displayed, this button tells the guide to display the previous day's content. */
    GuidePreviousDay,
    /** Toggles the display of information about the currently selected content, program, or media. */
    Info,
    /** Tellls the device to perform an instant replay (typically some form of jumping back a short amount of time then playing it again, possibly but not usually in slow motion). */
    InstantReplay,
    /** Opens content liniked to the current program, if available and possible. */
    Link,
    /** Lists the current program. */
    ListProgram,
    /** Toggles a display listing currently available live content or programs. */
    LiveContent,
    /** Locks or unlocks the currently selected content or pgoram. */
    Lock,
    /** Presents a list of media applications, such as photo viewers, audio and video players, and games. [1] */
    MediaApps,
    /** The Audio Track key. */
    MediaAudioTrack,
    /** Jumps back to the last-viewed content, program, or other media. */
    MediaLast,
    /** Skips backward to the previous content or program. */
    MediaSkipBackward,
    /** Skips forward to the next content or program. */
    MediaSkipForward,
    /** Steps backward to the previous content or program. */
    MediaStepBackward,
    /** Steps forward to the next content or program. */
    MediaStepForward,
    /** Top Menu button; opens the media's main menu, such as on a DVD or Blu-Ray disc. */
    MediaTopMenu,
    /** Navigates into a submenu or option. */
    NavigateIn,
    /** Navigates to the next item. */
    NavigateNext,
    /** Navigates out of the current screen or menu. */
    NavigateOut,
    /** Navigates to the previous item. */
    NavigatePrevious,
    /** Cycles to the next channel in the favorites list. */
    NextFavoriteChannel,
    /** Cycles to the next saved user profile, if this feature is supported and multiple profiles exist. */
    NextUserProfile,
    /** Opens the user interface for selecting on demand content or programs to watch. */
    OnDemand,
    /** Starts the process of pairing the remote with a device to be controlled. */
    Pairing,
    /** A button to move the picture-in-picture view downward. */
    PinPDown,
    /** A button to control moving the picture-in-picture view. */
    PinPMove,
    /** Toggles display of th epicture-in-picture view on and off. */
    PinPToggle,
    /** A button to move the picture-in-picture view upward. */
    PinPUp,
    /** Decreases the media playback rate. */
    PlaySpeedDown,
    /** Returns the media playback rate to normal. */
    PlaySpeedReset,
    /** Increases the media playback rate. */
    PlaySpeedUp,
    /** Toggles random media (also known as "shuffle mode") on and off. */
    RandomToggle,
    /** A code sent when the remote control's battery is low. This doesn't actually correspond to a physical key at all. */
    HandleLowBattery,
    /** Cycles among the available media recording speeds. */
    RecordSpeedNext,
    /** Toggles radio frequency (RF) input bypass mode on and off. RF bypass mode passes RF input directly to the RF output without any processing or filtering. */
    RfBypass,
    /** Toggles the channel scan mode on and off; this is a mode which flips through channels automatically until the user stops the scan. */
    ScanChannelsToggle,
    /** Cycles through the available screen display modes. */
    ScreenModeNext,
    /** Toggles display of the device's settings screen on and off. */
    Settings,
    /** Toggles split screen display mode on and off. */
    SplitScreenToggle,
    /** Cycles among input modes on an external set-top box (STB). */
    STBInput,
    /** Toggles on and off an external STB. */
    STBPower,
    /** Toggles the display of subtitles on and off if they're available. */
    Subtitle,
    /** Toggles display of teletext, if available. */
    Teletext,
    /** Cycles through the available video modes. */
    VideoModeNext,
    /** Causes the device to identify itself in some fashion, such as by flashing a light, briefly changing the brightness of indicator lights, or emitting a tone. */
    Wink,
    /** Toggles between full-screen and scaled content display, or otherwise change the magnification level. */
    ZoomToggle,
    /** Presents a list of possible corrections for a word which was incorrectly identified. */
    SpeechCorrectionList,
    /** Toggles between dictation mode and command/control mode. This lets the speech engine know whether to interpret spoken words as input text or as commands. */
    SpeechInputToggle,
    /** Closes the current document or message. Must not exit the application. */
    Close,
    /** Creates a new document or message. */
    New,
    /** Opens an existing document or message. */
    Open,
    /** Prints the current document or message. */
    Print,
    /** Saves the current document or message. */
    Save,
    /** Starts spell checking the current document. */
    SpellCheck,
    /** Opens the user interface to forward a message. */
    MailForward,
    /** Opens the user interface to reply to a message. */
    MailReply,
    /** Sends the current message. */
    MailSend,
    /** The Calculator key, often labeled with an icon such as . This is often used as a generic application launcher key (APPCOMMAND_LAUNCH_APP2). */
    LaunchCalculator,
    /** The Calendar key, often labeled with an icon like . */
    LaunchCalendar,
    /** The Contacts key. */
    LaunchContacts,
    /** The Mail key. This is often displayed as . */
    LaunchMail,
    /** The Media Player key. */
    LaunchMediaPlayer,
    /** The Music Player key, often labeled with an icon such as . */
    LaunchMusicPlayer,
    /** The My Computer key on Windows keyboards. This is often used as a generic application launcher key (APPCOMMAND_LAUNCH_APP1). */
    LaunchMyComputer,
    /** The Phone key, to open the phone dialer application if one is present. */
    LaunchPhone,
    /** The Screen Saver key. */
    LaunchScreenSaver,
    /** The Spreadsheet key. This key may be labeled with an icon such as  or that of a specific spreadsheet application. */
    LaunchSpreadsheet,
    /** The Web Browser key. This key is frequently labeled with an icon such as  or the icon of a specific browser, depending on the device manufacturer. */
    LaunchWebBrowser,
    /** The WebCam key. Opens the webcam application. */
    LaunchWebCam,
    /** The Word Processor key. This may be an icon of a specific word processor application, or a generic document icon. */
    LaunchWordProcessor,
    /** The first generic application launcher button. */
    LaunchApplication1,
    /** The second generic application launcher button. */
    LaunchApplication2,
    /** The third generic application launcher button. */
    LaunchApplication3,
    /** The fourth generic application launcher button. */
    LaunchApplication4,
    /** The fifth generic application launcher button. */
    LaunchApplication5,
    /** The sixth generic application launcher button. */
    LaunchApplication6,
    /** The seventh generic application launcher button. */
    LaunchApplication7,
    /** The eighth generic application launcher button. */
    LaunchApplication8,
    /** The ninth generic application launcher button. */
    LaunchApplication9,
    /** The 10th generic application launcher button. */
    LaunchApplication10,
    /** The 11th generic application launcher button. */
    LaunchApplication11,
    /** The 12th generic application launcher button. */
    LaunchApplication12,
    /** The 13th generic application launcher button. */
    LaunchApplication13,
    /** The 14th generic application launcher button. */
    LaunchApplication14,
    /** The 15th generic application launcher button. */
    LaunchApplication15,
    /** The 16th generic application launcher button. */
    LaunchApplication16,
    /** Navigates to the previous content or page in the current Web view's history. */
    BrowserBack,
    /** Opens the user's list of bookmarks/favorites. */
    BrowserFavorites,
    /** Navigates to the next content or page in the current Web view's history. */
    BrowserForward,
    /** Navigates to the user's preferred home page. */
    BrowserHome,
    /** Refreshes the current page or contentl. */
    BrowserRefresh,
    /** Activates the user's preferred search engine or the search interface within their browser. */
    BrowserSearch,
    /** Stops loading the currently displayed Web view or content. */
    BrowserStop,
    /** The decimal point key (typically . or , depending on the region. In newer browsers, this value to simply be the character generated by the decimal key (one of those two characters). [1] */
    Decimal,
    /** The 11 key found on certain media numeric keypads. */
    Key11,
    /** The 12 key found on certain media numeric keypads. */
    Key12,
    /** The numeric keypad's multiplication key, *. */
    Multiply,
    /** The numeric keypad's addition key, +. */
    Add,
    /** The numeric keypad's division key, /. */
    Divide,
    /** The numeric keypad's subtraction key, -. */
    Subtract,
    /** The numeric keypad's places separator character (in the United States, this is a comma, but elsewhere it is frequently a period). */
    Separator,
}

#[derive(Default, Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyTextEvent {
    pub char: char,
}
implement_message!(KeyTextEvent);

impl MessageFromString for KeyTextEvent {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("key_text") {
            let values = command_parser.get_values_of("key_text");
            return Some(KeyTextEvent { char: values[0] }.as_boxed());
        }
        None
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct KeyEvent {
    pub code: Key,
    pub state: InputState,
}
implement_message!(KeyEvent);

impl MessageFromString for KeyEvent {
    fn from_command_parser(command_parser: sabi_commands::CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("key_pressed") {
            let values = command_parser.get_values_of("key_pressed");
            return Some(
                KeyEvent {
                    code: values[0],
                    state: InputState::Pressed,
                }
                .as_boxed(),
            );
        } else if command_parser.has("key_released") {
            let values = command_parser.get_values_of("key_released");
            return Some(
                KeyEvent {
                    code: values[0],
                    state: InputState::Released,
                }
                .as_boxed(),
            );
        } else if command_parser.has("key_just_pressed") {
            let values = command_parser.get_values_of("key_just_pressed");
            return Some(
                KeyEvent {
                    code: values[0],
                    state: InputState::JustPressed,
                }
                .as_boxed(),
            );
        } else if command_parser.has("key_just_released") {
            let values = command_parser.get_values_of("key_just_released");
            return Some(
                KeyEvent {
                    code: values[0],
                    state: InputState::JustReleased,
                }
                .as_boxed(),
            );
        }
        None
    }
}

impl Default for KeyEvent {
    #[inline]
    fn default() -> Self {
        Self {
            code: Key::Unidentified,
            state: InputState::Invalid,
        }
    }
}

impl Default for Key {
    #[inline]
    fn default() -> Self {
        Key::Unidentified
    }
}

impl FromStr for Key {
    type Err = Key;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "a" => Ok(Key::A),
            "b" => Ok(Key::B),
            "c" => Ok(Key::C),
            "d" => Ok(Key::D),
            "e" => Ok(Key::E),
            "f" => Ok(Key::F),
            "g" => Ok(Key::G),
            "h" => Ok(Key::H),
            "i" => Ok(Key::I),
            "j" => Ok(Key::J),
            "k" => Ok(Key::K),
            "l" => Ok(Key::L),
            "m" => Ok(Key::M),
            "n" => Ok(Key::N),
            "o" => Ok(Key::O),
            "p" => Ok(Key::P),
            "q" => Ok(Key::Q),
            "r" => Ok(Key::R),
            "s" => Ok(Key::S),
            "t" => Ok(Key::T),
            "u" => Ok(Key::U),
            "v" => Ok(Key::V),
            "w" => Ok(Key::W),
            "x" => Ok(Key::X),
            "y" => Ok(Key::Y),
            "z" => Ok(Key::Z),
            "1" => Ok(Key::Numpad1),
            "2" => Ok(Key::Numpad2),
            "3" => Ok(Key::Numpad3),
            "4" => Ok(Key::Numpad4),
            "5" => Ok(Key::Numpad5),
            "6" => Ok(Key::Numpad6),
            "7" => Ok(Key::Numpad7),
            "8" => Ok(Key::Numpad8),
            "9" => Ok(Key::Numpad9),
            "0" => Ok(Key::Numpad0),
            "f1" => Ok(Key::F1),
            "f2" => Ok(Key::F2),
            "f3" => Ok(Key::F3),
            "f4" => Ok(Key::F4),
            "f5" => Ok(Key::F5),
            "f6" => Ok(Key::F6),
            "f7" => Ok(Key::F7),
            "f8" => Ok(Key::F8),
            "f9" => Ok(Key::F9),
            "f10" => Ok(Key::F10),
            "f11" => Ok(Key::F11),
            "f12" => Ok(Key::F12),
            "f13" => Ok(Key::F13),
            "f14" => Ok(Key::F14),
            "f15" => Ok(Key::F15),
            "f16" => Ok(Key::F16),
            "f17" => Ok(Key::F17),
            "f18" => Ok(Key::F18),
            "f19" => Ok(Key::F19),
            "f20" => Ok(Key::F20),
            "up" => Ok(Key::ArrowUp),
            "down" => Ok(Key::ArrowDown),
            "left" => Ok(Key::ArrowLeft),
            "right" => Ok(Key::ArrowRight),
            "alt" => Ok(Key::Alt),
            "altgr" => Ok(Key::AltGraph),
            "ctrl" => Ok(Key::Control),
            "shift" => Ok(Key::Shift),
            _ => Err(Key::Unidentified),
        }
    }
}
