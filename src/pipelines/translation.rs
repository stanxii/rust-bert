// Copyright 2018-2020 The HuggingFace Inc. team.
// Copyright 2020 Marian Team Authors
// Copyright 2019-2020 Guillaume Becquin
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! # Translation pipeline
//! Translation based on the Marian encoder-decoder architecture
//! Include techniques such as beam search, top-k and nucleus sampling, temperature setting and repetition penalty.
//! Pre-trained and ready-to-use models are available by creating a configuration from the `Language` enum.
//! These models have been trained by the Opus-MT team from Language Technology at the University of Helsinki (Jörg Tiedemann, jorg.tiedemann@helsinki.fi)
//! The Rust model files are hosted by Hugging Face Inc (https://huggingface.co).
//! Customized Translation models can be loaded by creating a configuration from local files.
//! The dependencies will be downloaded to the user's home directory, under ~/.cache/.rustbert/{translation-model-name}
//!
//!
//! ```no_run
//!# fn main() -> failure::Fallible<()> {
//!# use rust_bert::pipelines::generation::LanguageGenerator;
//! use rust_bert::pipelines::translation::{TranslationModel, TranslationConfig, Language};
//! use tch::Device;
//! let mut translation_config =  TranslationConfig::new(Language::EnglishToFrench, Device::cuda_if_available());
//! let mut model = TranslationModel::new(translation_config)?;
//!
//! let input = ["This is a sentence to be translated"];
//!
//! let output = model.translate(&input);
//!# Ok(())
//!# }
//! ```
//!
//! Output: \
//! ```no_run
//!# let output =
//! "Il s'agit d'une phrase à traduire"
//!# ;
//!```

use crate::pipelines::generation::{MarianGenerator, GenerateConfig, LanguageGenerator};
use tch::Device;
use crate::common::resources::{Resource, RemoteResource};
use crate::marian::{MarianModelResources, MarianConfigResources, MarianVocabResources, MarianSpmResources};

/// Pretrained languages available for direct use
pub enum Language {
    EnglishToFrench,
    FrenchToEnglish,
}

struct RemoteTranslationResources;

impl RemoteTranslationResources {
    pub const ENGLISH2FRENCH: ((&'static str, &'static str), (&'static str, &'static str), (&'static str, &'static str), (&'static str, &'static str)) =
        (MarianModelResources::ENGLISH2FRENCH, MarianConfigResources::ENGLISH2FRENCH, MarianVocabResources::ENGLISH2FRENCH, MarianSpmResources::ENGLISH2FRENCH);

    pub const FRENCH2ENGLISH: ((&'static str, &'static str), (&'static str, &'static str), (&'static str, &'static str), (&'static str, &'static str)) =
        (MarianModelResources::FRENCH2ENGLISH, MarianConfigResources::FRENCH2ENGLISH, MarianVocabResources::FRENCH2ENGLISH, MarianSpmResources::FRENCH2ENGLISH);
}


/// # Configuration for text translation
/// Contains information regarding the model to load, mirrors the GenerationConfig, with a
/// different set of default parameters and sets the device to place the model on.
pub struct TranslationConfig {
    /// Model weights resource (default: pretrained BART model on CNN-DM)
    pub model_resource: Resource,
    /// Config resource (default: pretrained BART model on CNN-DM)
    pub config_resource: Resource,
    /// Vocab resource (default: pretrained BART model on CNN-DM)
    pub vocab_resource: Resource,
    /// Merges resource (default: pretrained BART model on CNN-DM)
    pub merges_resource: Resource,
    /// Minimum sequence length (default: 0)
    pub min_length: u64,
    /// Maximum sequence length (default: 20)
    pub max_length: u64,
    /// Sampling flag. If true, will perform top-k and/or nucleus sampling on generated tokens, otherwise greedy (deterministic) decoding (default: true)
    pub do_sample: bool,
    /// Early stopping flag indicating if the beam search should stop as soon as `num_beam` hypotheses have been generated (default: false)
    pub early_stopping: bool,
    /// Number of beams for beam search (default: 5)
    pub num_beams: u64,
    /// Temperature setting. Values higher than 1 will improve originality at the risk of reducing relevance (default: 1.0)
    pub temperature: f64,
    /// Top_k values for sampling tokens. Value higher than 0 will enable the feature (default: 0)
    pub top_k: u64,
    /// Top_p value for [Nucleus sampling, Holtzman et al.](http://arxiv.org/abs/1904.09751). Keep top tokens until cumulative probability reaches top_p (default: 0.9)
    pub top_p: f64,
    /// Repetition penalty (mostly useful for CTRL decoders). Values higher than 1 will penalize tokens that have been already generated. (default: 1.0)
    pub repetition_penalty: f64,
    /// Exponential penalty based on the length of the hypotheses generated (default: 1.0)
    pub length_penalty: f64,
    /// Number of allowed repetitions of n-grams. Values higher than 0 turn on this feature (default: 3)
    pub no_repeat_ngram_size: u64,
    /// Number of sequences to return for each prompt text (default: 1)
    pub num_return_sequences: u64,
    /// Device to place the model on (default: CUDA/GPU when available)
    pub device: Device,
}

impl TranslationConfig {
    /// Create a new `TranslationCondiguration` from an available language.
    ///
    /// # Arguments
    ///
    /// * `language` - `Language` enum value (e.g. `Language::EnglishToFrench`)
    /// * `device` - `Device` to place the model on (CPU/GPU)
    ///
    /// # Example
    ///
    /// ```no_run
    ///# fn main() -> failure::Fallible<()> {
    /// use rust_bert::pipelines::translation::{TranslationConfig, Language};
    /// use tch::Device;
    ///
    /// let translation_config =  TranslationConfig::new(Language::FrenchToEnglish, Device::cuda_if_available());
    ///# Ok(())
    ///# }
    /// ```
    ///
    pub fn new(language: Language, device: Device) -> TranslationConfig {
        let (model_resource, config_resource, vocab_resource, merges_resource) = match language {
            Language::EnglishToFrench => RemoteTranslationResources::ENGLISH2FRENCH,
            Language::FrenchToEnglish => RemoteTranslationResources::FRENCH2ENGLISH
        };
        let model_resource = Resource::Remote(RemoteResource::from_pretrained(model_resource));
        let config_resource = Resource::Remote(RemoteResource::from_pretrained(config_resource));
        let vocab_resource = Resource::Remote(RemoteResource::from_pretrained(vocab_resource));
        let merges_resource = Resource::Remote(RemoteResource::from_pretrained(merges_resource));
        TranslationConfig {
            model_resource,
            config_resource,
            vocab_resource,
            merges_resource,
            min_length: 0,
            max_length: 512,
            do_sample: false,
            early_stopping: false,
            num_beams: 6,
            temperature: 1.0,
            top_k: 50,
            top_p: 1.0,
            repetition_penalty: 1.0,
            length_penalty: 1.0,
            no_repeat_ngram_size: 0,
            num_return_sequences: 1,
            device,
        }
    }

    /// Create a new `TranslationCondiguration` from custom (e.g. local) resources.
    ///
    /// # Arguments
    ///
    /// * `model_resource` - `Resource` pointing to the model
    /// * `config_resource` - `Resource` pointing to the configuration
    /// * `vocab_resource` - `Resource` pointing to the vocabulary
    /// * `sentence_piece_resource` - `Resource` pointing to the sentence piece model of the source language
    /// * `device` - `Device` to place the model on (CPU/GPU)
    ///
    /// # Example
    ///
    /// ```no_run
    ///# fn main() -> failure::Fallible<()> {
    /// use rust_bert::pipelines::translation::TranslationConfig;
    /// use tch::Device;
    /// use rust_bert::resources::{Resource, LocalResource};
    /// use std::path::PathBuf;
    ///
    /// let config_resource = Resource::Local(LocalResource { local_path: PathBuf::from("path/to/config.json") });
    /// let model_resource = Resource::Local(LocalResource { local_path: PathBuf::from("path/to/model.ot") });
    /// let vocab_resource = Resource::Local(LocalResource { local_path: PathBuf::from("path/to/vocab.json") });
    /// let sentence_piece_resource = Resource::Local(LocalResource { local_path: PathBuf::from("path/to/spiece.model") });
    ///
    /// let translation_config =  TranslationConfig::new_from_resources(model_resource,
    ///                                            config_resource,
    ///                                            vocab_resource,
    ///                                            sentence_piece_resource,
    ///                                            Device::cuda_if_available());
    ///# Ok(())
    ///# }
    /// ```
    ///
    pub fn new_from_resources(model_resource: Resource,
                              config_resource: Resource,
                              vocab_resource: Resource,
                              sentence_piece_resource: Resource,
                              device: Device) -> TranslationConfig {
        TranslationConfig {
            model_resource,
            config_resource,
            vocab_resource,
            merges_resource: sentence_piece_resource,
            min_length: 0,
            max_length: 512,
            do_sample: false,
            early_stopping: false,
            num_beams: 6,
            temperature: 1.0,
            top_k: 50,
            top_p: 1.0,
            repetition_penalty: 1.0,
            length_penalty: 1.0,
            no_repeat_ngram_size: 0,
            num_return_sequences: 1,
            device,
        }
    }
}

/// # TranslationModel to perform translation
pub struct TranslationModel {
    model: MarianGenerator
}

impl TranslationModel {
    /// Build a new `TranslationModel`
    ///
    /// # Arguments
    ///
    /// * `translation_config` - `TranslationConfig` object containing the resource references (model, vocabulary, configuration), translation options and device placement (CPU/GPU)
    ///
    /// # Example
    ///
    /// ```no_run
    ///# fn main() -> failure::Fallible<()> {
    /// use rust_bert::pipelines::translation::{TranslationModel, TranslationConfig, Language};
    /// use tch::Device;
    ///
    /// let translation_config =  TranslationConfig::new(Language::FrenchToEnglish, Device::cuda_if_available());
    /// let mut summarization_model =  TranslationModel::new(translation_config)?;
    ///# Ok(())
    ///# }
    /// ```
    ///
    pub fn new(translation_config: TranslationConfig)
               -> failure::Fallible<TranslationModel> {
        let generate_config = GenerateConfig {
            model_resource: translation_config.model_resource,
            config_resource: translation_config.config_resource,
            merges_resource: translation_config.merges_resource,
            vocab_resource: translation_config.vocab_resource,
            min_length: translation_config.min_length,
            max_length: translation_config.max_length,
            do_sample: translation_config.do_sample,
            early_stopping: translation_config.early_stopping,
            num_beams: translation_config.num_beams,
            temperature: translation_config.temperature,
            top_k: translation_config.top_k,
            top_p: translation_config.top_p,
            repetition_penalty: translation_config.repetition_penalty,
            length_penalty: translation_config.length_penalty,
            no_repeat_ngram_size: translation_config.no_repeat_ngram_size,
            num_return_sequences: translation_config.num_return_sequences,
            device: translation_config.device,
        };

        let model = MarianGenerator::new(generate_config)?;

        Ok(TranslationModel { model })
    }

    /// Translates texts provided
    ///
    /// # Arguments
    ///
    /// * `input` - `&[&str]` Array of texts to summarize.
    ///
    /// # Returns
    /// * `Vec<String>` Translated texts
    ///
    /// # Example
    ///
    /// ```no_run
    ///# fn main() -> failure::Fallible<()> {
    /// use rust_bert::pipelines::generation::LanguageGenerator;
    /// use rust_bert::pipelines::translation::{TranslationModel, TranslationConfig, Language};
    /// use tch::Device;
    ///
    /// let translation_config =  TranslationConfig::new(Language::EnglishToFrench, Device::cuda_if_available());
    /// let mut model = TranslationModel::new(translation_config)?;
    ///
    /// let input = ["This is a sentence to be translated"];
    ///
    /// let output = model.translate(&input);
    ///# Ok(())
    ///# }
    /// ```
    ///
    pub fn translate(&mut self, texts: &[&str]) -> Vec<String> {
        self.model.generate(Some(texts.to_vec()), None)
    }
}