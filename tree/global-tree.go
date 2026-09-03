package main

import (
	"context"
	"crypto/md5"
	"encoding/json"
	"fmt"
	"iter"
	"log/slog"
	"slices"
	"strings"

	_ "embed"
)

type GlobalTree struct {
	SVG           string                                    `json:"svg"`
	SVGHash       string                                    `json:"svgHash"`
	SVGWidth      float64                                   `json:"svgWidth"`
	SVGHeight     float64                                   `json:"svgHeight"`
	Elements      map[int]mermaidCompilationResponseElement `json:"elements"`
	MermaidConfig mermaidInitializeConfig                   `json:"mermaidConfig"`
}

var treeBackgroundColors = map[int]string{
	2021: "#b9291b",
	2022: "#00c0c6",
	2023: "#a05fdd",
	2024: "#48e675",
	2025: "#e67e22",
	2026: "#f1c40f",
}

type PeopleSet map[int64]struct{}
type RelationGraphUser struct {
	ID        int64     `json:"id"`
	Promotion int       `json:"promotion"`
	Parrains  PeopleSet `json:"parrains"`
	Filleuls  PeopleSet `json:"filleuls"`
}

type RelationsGraph struct {
	MinGen int                          `json:"minGen"`
	MaxGen int                          `json:"maxGen"`
	Users  map[int64]*RelationGraphUser `json:"users"`
}

const studentNodeClass = "studentnode"

func BuildRelationsGraph(BaseRelationsPTR *[]Parrainage) *RelationsGraph {
	BaseRelations := *BaseRelationsPTR
	if BaseRelations == nil {
		BaseRelations = []Parrainage{}
	}
	g := &RelationsGraph{
		MinGen: 20000,
		MaxGen: 0,
		Users:  make(map[int64]*RelationGraphUser),
	}

	// get min and max gen from allUsersMap
	for _, u := range allUsersMap {
		if u.Promotion < g.MinGen {
			g.MinGen = u.Promotion
		}
		if u.Promotion > g.MaxGen {
			g.MaxGen = u.Promotion
		}
	}

	for i := range BaseRelations {
		r := &BaseRelations[i]

		if r.ParrainID == 0 || r.FilleulID == 0 {
			continue
		}

		if _, ok := g.Users[r.ParrainID]; !ok {
			u, ok := allUsersMap[r.ParrainID]
			if ok {
				g.Users[r.ParrainID] = &RelationGraphUser{
					ID:        r.ParrainID,
					Promotion: u.Promotion,
					Parrains:  PeopleSet{},
					Filleuls:  PeopleSet{},
				}

			} else {
				slog.With("parrainID", r.ParrainID).Error("parrain not found in allUsersMap")
			}
		}

		if _, ok := g.Users[r.FilleulID]; !ok {
			u, ok := allUsersMap[r.FilleulID]
			if ok {
				g.Users[r.FilleulID] = &RelationGraphUser{
					ID:        r.FilleulID,
					Promotion: u.Promotion,
					Parrains:  map[int64]struct{}{},
					Filleuls:  map[int64]struct{}{},
				}

				if u.Promotion < g.MinGen {
					g.MinGen = u.Promotion
				}
				if u.Promotion > g.MaxGen {
					g.MaxGen = u.Promotion
				}
			} else {
				slog.With("filleulID", r.FilleulID).Error("filleul not found in allUsersMap")
			}
		}

		if g.Users[r.ParrainID] != nil && g.Users[r.FilleulID] != nil {
			g.Users[r.ParrainID].Filleuls[r.FilleulID] = struct{}{}
			g.Users[r.FilleulID].Parrains[r.ParrainID] = struct{}{}
		}
	}

	/* filename := fmt.Sprintf("./%d-%d-%d.tree.json", time.Now().Year(), time.Now().Month(), time.Now().Day())
	file, err := os.Create(filename)
	if err != nil {
		slog.With("error", err).Error("Failed to create file")
		return nil
	}
	defer file.Close()
	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	err = encoder.Encode(g)
	if err != nil {
		slog.With("error", err).Error("Failed to encode graph to JSON")
		return nil
	}
	*/

	return g

}

func GenerateMermaidCodeFromGraph(graph *RelationsGraph) (string, string, error) {
	if graph == nil {
		return "", "", fmt.Errorf("failed to build relations graph")
	}

	maxPromotion := graph.MaxGen
	minPromotion := graph.MinGen

	type BucketItem []*RelationGraphUser
	buckets := make(map[int]BucketItem, maxPromotion-minPromotion+1)
	for i := minPromotion; i <= maxPromotion; i++ {
		buckets[i] = make(BucketItem, 0)
	}

	// Make sure to use bucketIter() to iterate over buckets, as it guarantees the order
	bucketIter := func() iter.Seq2[int, BucketItem] {
		return func(yield func(int, BucketItem) bool) {
			for i := minPromotion; i <= maxPromotion; i++ {
				if !yield(i, buckets[i]) {
					return
				}
			}
		}
	}

	for _, u := range graph.Users {
		// if len(u.Filleuls) == 0 && len(u.Parrains) == 0 {
		// 	continue
		// }
		buckets[u.Promotion] = append(buckets[u.Promotion], u)
	}

	for i := range bucketIter() {
		slices.SortFunc(buckets[i], func(u1, u2 *RelationGraphUser) int {
			return int(u1.ID) - int(u2.ID)
		})
	}

	// Now, generate tree
	tree := `
graph TD;
`

	for p, us := range bucketIter() {
		if len(us) == 0 {
			continue
		}
		tree += fmt.Sprintf("	subgraph Gen%d[\"Génération %d\"]\n", p, p)
		for _, u := range us {
			tree += fmt.Sprintf(" 	   %d(\"%q\")\n", u.ID, allUsersMap[u.ID].FirstName)
		}
		tree += "	end\n"
	}

	for _, us := range bucketIter() {
		for _, u := range us {
			if len(u.Filleuls) == 0 {
				continue
			}
			fi := make([]string, 0, len(u.Filleuls))
			for f := range u.Filleuls {
				if _, ok := graph.Users[f]; !ok {
					continue
				}
				fi = append(fi, fmt.Sprintf("%d:::%s", f, studentNodeClass))
			}
			tree += fmt.Sprintf("    %d:::%s --> %s\n", u.ID, studentNodeClass, strings.Join(fi, " & "))
		}
	}

	tree += "	\n"

	for p := range bucketIter() {
		if color, ok := treeBackgroundColors[p]; ok {
			tree += fmt.Sprintf(`   style Gen%d fill:%s`, p, color) + "\n"
		}
	}

	tree += "	\n"

	for _, us := range bucketIter() {
		for _, u := range us {
			tree += fmt.Sprintf("	click %d call nodeClicked() \"Voir graphe spécifique de %s\"\n", u.ID, allUsersMap[u.ID].FirstName)
		}
	}

	tree += "	\n"

	return tree, fmt.Sprintf("%x", md5.Sum([]byte(tree))), nil //nolint:gosec
}

// code, hash, err
func GenerateMermaidCode() (string, string, error) {
	// Refresh users cache
	_, err := usersCacher.Get()
	if err != nil {
		slog.With("error", err).Error("Failed to get users")
		return "", "", err
	}
	cachedRelations, err := relationsCacher.Get()
	if err != nil {
		slog.With("error", err).Error("Failed to get parrainages")
		return "", "", err
	}

	return GenerateMermaidCodeFromGraph(cachedRelations.Graph)
}

// https://github.com/abhinav/goldmark-mermaid?tab=readme-ov-file
//
//go:embed mermaid.min.js
var mermaidJSSource string

//go:embed extraMermaidCode.js
var mermaidExtraJS string

type mermaidCompilationResponseElement struct {
	X float64 `json:"x"`
	Y float64 `json:"y"`
}

type mermaidCompilationResponse struct {
	SVG       string                                    `json:"SVG"`
	SVGWidth  float64                                   `json:"svgWidth"`
	SVGHeight float64                                   `json:"svgHeight"`
	Elements  map[int]mermaidCompilationResponseElement `json:"elements"`
}

type mermaidElk struct {
	MergeEdges            bool   `json:"mergeEdges"`
	NodePlacementStrategy string `json:"nodePlacementStrategy"`
}

// Configuration for mermaid.initialize.
// Maps to MermaidConfig.
type mermaidInitializeConfig struct {
	// Theme to use for rendering.
	//
	// Values include "dark", "default", "forest", and "neutral".
	// See MermaidJS documentation for a full list.
	Theme          string            `json:"theme,omitempty"`
	StartOnLoad    bool              `json:"startOnLoad,omitempty"`
	Layout         string            `json:"layout,omitempty"`
	Elk            mermaidElk        `json:"elk,omitempty"`
	ThemeVariables map[string]string `json:"themeVariables,omitempty"`
}

var MermaidInitConfig = mermaidInitializeConfig{
	Theme:       "dark",
	StartOnLoad: false,
	Layout:      "elk",
	Elk: mermaidElk{
		MergeEdges:            true,
		NodePlacementStrategy: "SIMPLE",
	},
	ThemeVariables: map[string]string{
		"lineColor": "white",
	},
}

func GetMermaidCompiler() (*ChromeCompiler[string, mermaidCompilationResponse], error) {
	// Build the initialization code.
	var init strings.Builder
	init.WriteString("mermaid.initialize(")
	if err := json.NewEncoder(&init).Encode(MermaidInitConfig); err != nil {
		panic(err)
	}
	init.WriteString(")")

	// Create the compiler.
	compiler, err := CreateChromeCompiler(&ChromeCompilerConfig[string, mermaidCompilationResponse]{
		NoSandbox: true,
		JSSource:  mermaidJSSource,
		JSInit:    init.String(),
		JSExtra:   mermaidExtraJS,
		OutputProcessor: func(s *string) (*mermaidCompilationResponse, error) {
			var resp mermaidCompilationResponse
			if err := json.Unmarshal([]byte(*s), &resp); err != nil {
				return nil, err
			}
			return &resp, nil
		},
	})
	if err != nil {
		slog.With("error", err).Error("error creating compiler")
		return nil, err
	}
	return compiler, nil
}

func RunGenerator() (GlobalTree, error) {
	mermaidCode, mermaidCodeHash, err := GenerateMermaidCode()
	if err != nil {
		slog.With("error", err).Error("error generating mermaid code")
		return GlobalTree{}, err
	}

	compiler, err := GetMermaidCompiler()
	if err != nil {
		slog.With("error", err).Error("error creating compiler")
		return GlobalTree{}, err
	}
	defer compiler.Close()

	ctx := context.Background()
	compiled, err := compiler.Compile(
		ctx,
		"renderSVG",
		&mermaidCode,
	)

	if err != nil {
		slog.With("error", err).Error("error compiling")
		return GlobalTree{}, err
	}

	return GlobalTree{
		SVG:           compiled.SVG,
		SVGHash:       mermaidCodeHash,
		SVGWidth:      compiled.SVGWidth,
		SVGHeight:     compiled.SVGHeight,
		Elements:      compiled.Elements,
		MermaidConfig: MermaidInitConfig,
	}, nil
}

func CommonKeys[K comparable, V1 any, V2 any](map1 map[K]V1, map2 map[K]V2) map[K]V1 {
	result := make(map[K]V1)
	for k, v := range map1 {
		if _, exists := map2[k]; exists {
			result[k] = v
		}
	}
	return result
}

// ExtractUserGraph extracts the entire downward tree (all filleuls recursively),
// the entire upward lineage (all parrains recursively), co-filleuls, and any
// cross-links (e.g. if a co-filleul is a parrain to a filleul or a filleul to a parrain).
func ExtractUserGraph(userID int64, baseGraph *RelationsGraph) (*RelationsGraph, error) {
	me, ok := baseGraph.Users[userID]
	if !ok {
		return nil, fmt.Errorf("user %d not found in graph", userID)
	}

	g := &RelationsGraph{
		MinGen: baseGraph.MinGen,
		MaxGen: baseGraph.MaxGen,
		Users:  make(map[int64]*RelationGraphUser),
	}

	copyUser := func(src *RelationGraphUser) *RelationGraphUser {
		newUser := &RelationGraphUser{
			ID:        src.ID,
			Promotion: src.Promotion,
		}
		if src.Parrains != nil {
			newUser.Parrains = make(PeopleSet, len(src.Parrains))
			for k, v := range src.Parrains {
				newUser.Parrains[k] = v
			}
		}
		if src.Filleuls != nil {
			newUser.Filleuls = make(PeopleSet, len(src.Filleuls))
			for k, v := range src.Filleuls {
				newUser.Filleuls[k] = v
			}
		}
		return newUser
	}

	// 1. Add requested target user
	g.Users[userID] = copyUser(me)

	// 2. Extract ALL Filleuls Downward (Recursive / Iterative BFS)
	filleulQueue := []int64{userID}
	for len(filleulQueue) > 0 {
		currID := filleulQueue[0]
		filleulQueue = filleulQueue[1:]

		currUser, ok := baseGraph.Users[currID]
		if !ok {
			continue
		}

		for filleulID := range currUser.Filleuls {
			if _, exists := g.Users[filleulID]; !exists {
				if filleul, ok := baseGraph.Users[filleulID]; ok {
					g.Users[filleulID] = copyUser(filleul)
					filleulQueue = append(filleulQueue, filleulID)
				}
			}
		}
	}

	// 3. Extract ALL Parrains Upward (Recursive / Iterative BFS)
	parrainQueue := []int64{userID}
	for len(parrainQueue) > 0 {
		currID := parrainQueue[0]
		parrainQueue = parrainQueue[1:]

		currUser, ok := baseGraph.Users[currID]
		if !ok {
			continue
		}

		for parrainID := range currUser.Parrains {
			if _, exists := g.Users[parrainID]; !exists {
				if parrain, ok := baseGraph.Users[parrainID]; ok {
					g.Users[parrainID] = copyUser(parrain)
					parrainQueue = append(parrainQueue, parrainID)
				}
			}
		}
	}

	// 4. Extract Co-Filleuls (Same-promotion siblings from direct parrains)
	for parrainID := range me.Parrains {
		parrain, ok := baseGraph.Users[parrainID]
		if !ok {
			continue
		}

		for coFilleulID := range parrain.Filleuls {
			if _, exists := g.Users[coFilleulID]; exists {
				continue
			}
			coFilleul, ok := baseGraph.Users[coFilleulID]
			if !ok || coFilleul.Promotion != me.Promotion {
				continue
			}

			g.Users[coFilleulID] = copyUser(coFilleul)
		}
	}

	// 5. Cross-Link Pass: Connect any missing edges between extracted nodes.
	// This ensures that if node A (e.g. co-filleul) has a relationship with node B
	// (e.g. a downstream filleul), the link is preserved in g.Users.
	for extractedID, userNode := range g.Users {
		baseNode, ok := baseGraph.Users[extractedID]
		if !ok {
			continue
		}

		// Ensure all parrains of extracted users that exist in g are linked
		for parrainID := range baseNode.Parrains {
			if _, exists := g.Users[parrainID]; exists {
				if userNode.Parrains == nil {
					userNode.Parrains = make(PeopleSet)
				}
				userNode.Parrains[parrainID] = baseNode.Parrains[parrainID]
			}
		}

		// Ensure all filleuls of extracted users that exist in g are linked
		for filleulID := range baseNode.Filleuls {
			if _, exists := g.Users[filleulID]; exists {
				if userNode.Filleuls == nil {
					userNode.Filleuls = make(PeopleSet)
				}
				userNode.Filleuls[filleulID] = baseNode.Filleuls[filleulID]
			}
		}
	}

	return g, nil
}
