//! CSS Speech values.

pub(crate) mod values;

#[doc(inline)]
pub use values::{
    CssAudioCue, CssCue, CssDecibelLiteral, CssGenericVoice, CssSemitoneLiteral, CssSpeak,
    CssSpeakAs, CssSpeakAsPunctuation, CssSpeechBreak, CssSpeechBreakStrength, CssVoiceAge,
    CssVoiceBalance, CssVoiceBalanceKeyword, CssVoiceDuration, CssVoiceFamily, CssVoiceFamilyEntry,
    CssVoiceFamilyList, CssVoiceFamilyName, CssVoiceFamilyNameRef, CssVoiceGender, CssVoiceLevel,
    CssVoiceOffset, CssVoicePitchRange, CssVoiceRate, CssVoiceRateKeyword, CssVoiceStress,
    CssVoiceVolume, CssVoiceVolumeLevel,
};
