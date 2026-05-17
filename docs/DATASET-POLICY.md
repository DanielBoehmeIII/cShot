# Dataset Policy — cShot

cShot does not use any external datasets for its local engine generation.

## cShot Engine

The default provider (`cshot-engine`) generates audio algorithmically:
- Sine oscillators with envelope shaping
- White/pink noise with filtering
- DSP transforms (pitch shift, time stretch, saturation)

No training data, no datasets, no model weights.

## Third-Party Providers

If you configure a real provider (ElevenLabs, Stable Audio, etc.), that
provider may use its own training data. cShot does not control or audit
third-party training datasets.

For provider-specific dataset information, refer to each provider's
documentation and licensing terms.
