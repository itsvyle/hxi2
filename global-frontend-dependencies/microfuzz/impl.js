"use strict";

exports.__esModule = true;
exports.aggressiveFuzzyMatch = aggressiveFuzzyMatch;
exports.createFuzzySearchImpl = createFuzzySearchImpl;
exports.experimentalSmartFuzzyMatch = experimentalSmartFuzzyMatch;
exports.fuzzyMatchImpl = fuzzyMatchImpl;
var _normalizeText = _interopRequireDefault(require("./normalizeText"));
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : { default: obj };
}
// @flow
/* eslint-disable no-continue */
/*:: import type {
  Range,
  FuzzySearcher,
  FuzzySearchOptions,
  FuzzySearchStrategy,
  FuzzyResult,
  HighlightRanges,
  FuzzyMatches,
} from './index'*/
var MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
var sortByScore = function sortByScore(
    a /*: FuzzyResult<T>*/,
    b /*: FuzzyResult<T>*/,
) {
    return /*: number*/ a.score - b.score;
}; /*:: <T>*/
var sortRangeTuple = function sortRangeTuple(a /*: Range*/, b /*: Range*/) {
    return /*: number*/ a[0] - b[0];
};
var validWordBoundaries = new Set("  []()-–—'\"“”".split(""));
function isValidWordBoundary(character /*: string*/) /*: boolean*/ {
    return validWordBoundaries.has(character);
}
function matchesFuzzily(
    item /*: string*/,
    normalizedItem /*: string*/,
    itemWords /*: Set<string>*/,
    query /*: string*/,
    normalizedQuery /*: string*/,
    queryWords /*: string[]*/,
    strategy /*: FuzzySearchStrategy*/,
) /*: ?[number, HighlightRanges]*/ {
    // quick matches
    if (item === query) {
        return [0, [[0, item.length - 1]]];
    }
    var queryLen = query.length;
    var normalizedItemLen = normalizedItem.length;
    var normalizedQueryLen = normalizedQuery.length;
    if (normalizedItem === normalizedQuery) {
        return [0.1, [[0, normalizedItemLen - 1]]];
    } else if (normalizedItem.startsWith(normalizedQuery)) {
        return [0.5, [[0, normalizedQueryLen - 1]]];
    }

    // contains query (starting at word boundary)
    // NOTE: It would be more correct to do a regex search, than to check previous character, since
    // it could be that the item found does _not_ start at a word boundary, but there is another match
    // that does. However, this is faster and should rarely be a problem, while fuzzy search will still
    // find other matches (just ranked lower)
    var exactContainsIdx = item.indexOf(query);
    if (
        exactContainsIdx > -1 &&
        isValidWordBoundary(item[exactContainsIdx - 1])
    ) {
        return [0.9, [[exactContainsIdx, exactContainsIdx + queryLen - 1]]];
    }
    var containsIdx = normalizedItem.indexOf(normalizedQuery);
    if (
        containsIdx > -1 &&
        isValidWordBoundary(normalizedItem[containsIdx - 1])
    ) {
        return [1, [[containsIdx, containsIdx + queryLen - 1]]];
    }

    // Match by words included
    // Score: 1.5 + 0.2*words (so that it's better than two non-word chunks)
    var queryWordCount = queryWords.length;
    if (queryWordCount > 1) {
        if (
            queryWords.every(function (word) {
                return itemWords.has(word);
            })
        ) {
            var score = 1.5 + queryWordCount * 0.2;
            return [
                score,
                queryWords
                    .map(function (word) {
                        var wordIndex = normalizedItem.indexOf(word);
                        return [
                            wordIndex,
                            wordIndex + word.length - 1,
                        ] /*: Range*/;
                    })
                    .sort(sortRangeTuple),
            ];
        }
    }

    // Contains query (at any position)
    if (containsIdx > -1) {
        return [2, [[containsIdx, containsIdx + queryLen - 1]]];
    }

    // Match by consecutive letters (fuzzy)
    if (strategy === "aggressive") {
        return aggressiveFuzzyMatch(normalizedItem, normalizedQuery);
    } else if (strategy === "smart") {
        return experimentalSmartFuzzyMatch(normalizedItem, normalizedQuery);
    }
    return null;
}
function aggressiveFuzzyMatch(
    normalizedItem /*: string*/,
    normalizedQuery /*: string*/,
) /*: ?[number, HighlightRanges]*/ {
    var normalizedItemLen = normalizedItem.length;
    var normalizedQueryLen = normalizedQuery.length;
    var queryIdx = 0;
    var queryChar = normalizedQuery[queryIdx];
    var indices /*: HighlightRanges*/ = [];
    var chunkFirstIdx = -1;
    var chunkLastIdx = -2;
    // TODO: May improve performance by early exits (less to go than remaining query)
    // and by using .indexOf(x, fromIndex)
    for (var itemIdx = 0; itemIdx < normalizedItemLen; itemIdx += 1) {
        // DEBUG:
        // console.log(`${itemIdx} (${normalizedItem[itemIdx]}), ${queryIdx} (${queryChar}), ${chunkLastIdx}, score: ${consecutiveChunks}`)
        if (normalizedItem[itemIdx] === queryChar) {
            if (itemIdx !== chunkLastIdx + 1) {
                if (chunkFirstIdx >= 0) {
                    indices.push([chunkFirstIdx, chunkLastIdx]);
                }
                chunkFirstIdx = itemIdx;
            }
            chunkLastIdx = itemIdx;
            queryIdx += 1;
            if (queryIdx === normalizedQueryLen) {
                indices.push([chunkFirstIdx, chunkLastIdx]);
                return scoreConsecutiveLetters(indices, normalizedItem);
            }
            queryChar = normalizedQuery[queryIdx];
        }
    }
    return null;
}
function experimentalSmartFuzzyMatch(
    normalizedItem /*: string*/,
    normalizedQuery /*: string*/,
) /*: ?[number, HighlightRanges]*/ {
    var normalizedItemLen = normalizedItem.length;

    // Match by consecutive letters, but only match beginnings of words or chunks of 3+ letters
    // Note that there may be multiple valid ways in which such matching can be done, and we'll only
    // match each chunk to the first one found that matches these criteria. It's not perfect as it's
    // possible that later chunks will fail to match while there's a better match, for example:
    // - query: ABC
    // - item: A xABC
    //         ^___xx (no match)
    //         ___^^^ (better match)
    // But we want to limit the algorithmic complexity and this should generally work.

    var indices /*: HighlightRanges*/ = [];
    var queryIdx = 0;
    var queryChar = normalizedQuery[queryIdx];
    var chunkFirstIdx = -1;
    var chunkLastIdx = -2;

    // eslint-disable-next-line no-constant-condition
    while (true) {
        // Find match for first letter of chunk
        var idx = normalizedItem.indexOf(queryChar, chunkLastIdx + 1);
        if (idx === -1) {
            break;
        }

        // Check if chunk starts at word boundary
        if (idx === 0 || isValidWordBoundary(normalizedItem[idx - 1])) {
            chunkFirstIdx = idx;
        } else {
            // Else, check if chunk is at least 3+ letters
            var queryCharsLeft = normalizedQuery.length - queryIdx;
            var itemCharsLeft = normalizedItem.length - idx;
            var minimumChunkLen = Math.min(3, queryCharsLeft, itemCharsLeft);
            var minimumQueryChunk = normalizedQuery.slice(
                queryIdx,
                queryIdx + minimumChunkLen,
            );
            if (
                normalizedItem.slice(idx, idx + minimumChunkLen) ===
                minimumQueryChunk
            ) {
                chunkFirstIdx = idx;
            } else {
                // Move index to continue search for valid chunk
                chunkLastIdx += 1;
                continue;
            }
        }

        // We have first index of a valid chunk, find its last index
        // TODO: We could micro-optimize by setting chunkLastIdx earlier if we already know it's len 3 or more
        for (
            chunkLastIdx = chunkFirstIdx;
            chunkLastIdx < normalizedItemLen;
            chunkLastIdx += 1
        ) {
            if (normalizedItem[chunkLastIdx] !== queryChar) {
                break;
            }
            queryIdx += 1;
            queryChar = normalizedQuery[queryIdx];
        }

        // Add chunk to indices
        chunkLastIdx -= 1; // decrement as we've broken out of loop on non-matching char
        indices.push([chunkFirstIdx, chunkLastIdx]);

        // Check if we're done
        if (queryIdx === normalizedQuery.length) {
            return scoreConsecutiveLetters(indices, normalizedItem);
        }
    }

    // eslint-disable-next-line no-unreachable
    return null;
}
function scoreConsecutiveLetters(
    indices /*: HighlightRanges*/,
    normalizedItem /*: string*/,
) /*: ?[number, HighlightRanges]*/ {
    // Score: 2 + sum of chunk scores
    // Chunk scores:
    // - 0.2 for a full word
    // - 0.4 for chunk starting at beginning of word
    // - 0.8 for chunk in the middle of the word (if >=3 characters)
    // - 1.6 for chunk in the middle of the word (if 1 or 2 characters)
    var score = 2;
    indices.forEach(function (_ref) {
        var firstIdx = _ref[0],
            lastIdx = _ref[1];
        var chunkLength = lastIdx - firstIdx + 1;
        var isStartOfWord =
            firstIdx === 0 ||
            normalizedItem[firstIdx] === " " ||
            normalizedItem[firstIdx - 1] === " ";
        var isEndOfWord =
            lastIdx === normalizedItem.length - 1 ||
            normalizedItem[lastIdx] === " " ||
            normalizedItem[lastIdx + 1] === " ";
        var isFullWord = isStartOfWord && isEndOfWord;
        // DEBUG:
        // console.log({
        //   firstIdx,
        //   lastIdx,
        //   chunkLength,
        //   isStartOfWord,
        //   isEndOfWord,
        //   isFullWord,
        //   before: normalizedItem[firstIdx - 1],
        //   after: normalizedItem[lastIdx + 1],
        // })
        if (isFullWord) {
            score += 0.2;
        } else if (isStartOfWord) {
            score += 0.4;
        } else if (chunkLength >= 3) {
            score += 0.8;
        } else {
            score += 1.6;
        }
    });
    return [score, indices];
}
function fuzzyMatchImpl(
    text /*: string*/,
    query /*: string*/,
) /*: ?FuzzyResult<string>*/ {
    var normalizedQuery = (0, _normalizeText["default"])(query);
    var queryWords = normalizedQuery.split(" ");
    var normalizedText = (0, _normalizeText["default"])(text);
    var itemWords = new Set(normalizedText.split(" "));
    var result = matchesFuzzily(
        text,
        normalizedText,
        itemWords,
        query,
        normalizedQuery,
        queryWords,
        "smart",
    );
    if (result) {
        return {
            item: text,
            score: result[0],
            matches: [result[1]],
        };
    }
    return null;
}
function createFuzzySearchImpl /*:: <Element>*/(
    collection /*: Element[]*/,
    options /*: FuzzySearchOptions*/,
) /*: FuzzySearcher<Element>*/ {
    // TODO: Change default strategy to smart
    var _options$strategy = options.strategy,
        strategy =
            _options$strategy === void 0 ? "aggressive" : _options$strategy,
        getText = options.getText;
    var preprocessedCollection /*: [Element, [string, string, Set<string>][]][]*/ =
        collection.map(function (element /*: Element*/) {
            var texts /*: (?string)[]*/;
            if (getText) {
                texts = getText(element);
            } else {
                // $FlowFixMe[incompatible-use]
                var text /*: string*/ = options.key
                    ? element[options.key]
                    : element; /*: any*/
                texts = [text];
            }
            var preprocessedTexts /*: [string, string, Set<string>][]*/ =
                texts.map(function (text) {
                    var item = text || "";
                    var normalizedItem = (0, _normalizeText["default"])(item);
                    var itemWords = new Set(normalizedItem.split(" "));
                    return [item, normalizedItem, itemWords];
                });
            return [element, preprocessedTexts];
        });
    return function (query /*: string*/) {
        // DEBUG
        // const b4 = Date.now()
        var results /*: Array<FuzzyResult<Element>>*/ = [];
        var normalizedQuery = (0, _normalizeText["default"])(query);
        var queryWords = normalizedQuery.split(" ");
        if (!normalizedQuery.length) {
            return [];
        }
        preprocessedCollection.forEach(function (_ref2) {
            var element = _ref2[0],
                texts = _ref2[1];
            var bestScore = MAX_SAFE_INTEGER;
            var matches /*: FuzzyMatches*/ = [];
            for (var i = 0, len = texts.length; i < len; i += 1) {
                var _texts$i = texts[i],
                    item = _texts$i[0],
                    normalizedItem = _texts$i[1],
                    itemWords = _texts$i[2];
                var result = matchesFuzzily(
                    item,
                    normalizedItem,
                    itemWords,
                    query,
                    normalizedQuery,
                    queryWords,
                    strategy,
                );
                if (result) {
                    bestScore = Math.min(bestScore, result[0]); // take the lowest score of any match
                    matches.push(result[1]);
                } else {
                    matches.push(null);
                }
            }
            if (bestScore < MAX_SAFE_INTEGER) {
                results.push({
                    item: element,
                    score: bestScore,
                    matches: matches,
                });
            }
        });
        results.sort(sortByScore);

        // DEBUG
        // console.log(`fuzzy search complete in ${Date.now() - b4} ms`)

        return results;
    };
}
