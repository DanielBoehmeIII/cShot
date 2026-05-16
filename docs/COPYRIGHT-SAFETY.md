# Copyright Safety — cShot

**Last updated:** 2026-05-16

This document explains cShot's approach to copyright safety. It is not legal
advice. Consult a lawyer for legal questions about your specific use case.

---

## 1. How cShot Generates Sounds

cShot generates sounds using:
- **Mock DSP synthesis**: algorithmic waveform generation (sine waves, noise,
  envelopes, filters). These are mathematical constructs, not reproductions
  of existing recordings.
- **Local audio processing**: trim, normalize, fade, EQ — applied to your
  uploaded reference or to synthesized sounds. These are standard DSP
  operations used in every DAW.

No generation uses models trained on unlicensed third-party samples.

---

## 2. Your Responsibility

As the user of cShot:

- **You own your prompts.** The text you type is your creative input.
- **You own your reference audio.** Any file you upload is your property.
  cShot does not store or transmit it.
- **You own the output.** Generated WAV files are yours to use in your
  projects, commercial or otherwise.
- **You are responsible for your reference audio.** If you upload a sample
  that is copyrighted and you do not have a license to use it, the generated
  derivative may inherit that legal encumbrance.

---

## 3. Reference Upload Warning

When you upload a reference WAV file:

- cShot analyzes it locally (RMS, spectral centroid, duration).
- The analysis data is used to shape the generated sound.
- The original reference file stays on your machine.
- No audio data is sent to any server unless you configure a cloud provider.

**Important:** If your reference audio contains copyrighted material you
don't have rights to, the generated sound may be considered a derivative
work. Use only your own recordings or royalty-free samples as reference.

---

## 4. Commercial Use

Generated sounds from the mock DSP pipeline are safe for commercial use.

If you integrate a third-party generation provider (ElevenLabs, Stable Audio,
etc.), review that provider's terms of service. Some providers claim ownership
of generated content or restrict commercial use at certain tiers.

---

## 5. Attribution

cShot does not require attribution for generated sounds. You are not required
to credit cShot in your releases or projects.

If you share generated sounds as sample packs, we appreciate a mention but
do not require one.

---

## 6. Model Safety

cShot's mock DSP provider:
- Does not use training data
- Does not reproduce existing recordings
- Does not memorize or interpolate copyrighted samples
- Produces unique output for every prompt/seed combination

This makes it the safest option for commercial production. Real AI providers
(if configured) may carry different risks — review their copyright stance
before using in commercial projects.

---

## 7. Limitations

cShot makes no guarantees about:
- Third-party model behavior (if you configure an external provider)
- Similarity to existing copyrighted works (algorithmic coincidence is
  possible with any sound generation)
- Legal outcomes of using generated audio in commercial projects

If in doubt, consult a lawyer.
