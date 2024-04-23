// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2024  Neil Roberts
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

package uk.co.busydoingnothing.pocketrl;

public class PartOfSpeech {
    private static String[] names = {
        "adjective", // AJ0
        "comparative adjective", // AJC
        "superlative adjective", // AJS
        "article", // AT0
        "adverb", // AV0
        "adverb participle", // AVP
        "wh-adverb", // AVQ
        "coordinating conjuction", // CJC
        "subordinating conjuction", // CJS
        "conjunction ‘that’", // CJT
        "cardinal number", // CRD
        "possessive determiner", // DPS
        "general determiner", // DT0
        "wh-determiner", // DTQ
        "existential ‘there’", // EX0
        "interjection", // ITJ
        "noun (neutral for number)", // NN0
        "singular noun", // NN1
        "plural noun", // NN2
        "proper noun", // NP0
        "ordinal", // ORD
        "indefinite pronoun", // PNI
        "personal pronoun", // PNP
        "wh-pronoun", // PNQ
        "reflexive pronoun", // PNX
        "the possessive", // POS
        "prefix", // PRE
        "the preposition ‘of’", // PRF
        "preposition", // PRP
        "the infinitive marker ‘to’", // TO0
        "unclassified", // UNC
        "verb modal auxiliary", // VM0
        "verb", // VVB
        "past tense", // VVD
        "present participle", // VVG
        "past participle", // VVN
        "3rd person singular", // VVZ
        "negative participle", // XX0
        "alphabetical symbol", // ZZ0
    };

    public static String name(int index)
    {
        if (index < names.length)
            return names[index];
        else
            return "unknown";
    }
}
