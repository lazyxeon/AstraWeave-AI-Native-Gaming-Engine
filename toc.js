// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting-started/quick-start.html"><strong aria-hidden="true">1.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">2.</strong> Installation Guide</a></li><li class="chapter-item expanded "><a href="getting-started/first-companion.html"><strong aria-hidden="true">3.</strong> Your First AI Companion</a></li><li class="chapter-item expanded "><a href="getting-started/requirements.html"><strong aria-hidden="true">4.</strong> System Requirements</a></li><li class="chapter-item expanded affix "><li class="part-title">Architecture</li><li class="chapter-item expanded "><a href="architecture/overview.html"><strong aria-hidden="true">5.</strong> Overview</a></li><li class="chapter-item expanded "><a href="architecture/ai-native.html"><strong aria-hidden="true">6.</strong> AI-Native Design</a></li><li class="chapter-item expanded "><a href="architecture/ecs.html"><strong aria-hidden="true">7.</strong> ECS Architecture</a></li><li class="chapter-item expanded "><a href="architecture/deterministic.html"><strong aria-hidden="true">8.</strong> Deterministic Simulation</a></li><li class="chapter-item expanded "><a href="architecture/tool-validation.html"><strong aria-hidden="true">9.</strong> Tool Validation System</a></li><li class="chapter-item expanded affix "><li class="part-title">Core Systems</li><li class="chapter-item expanded "><a href="core-systems/ai/index.html"><strong aria-hidden="true">10.</strong> AI System</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="core-systems/ai/perception.html"><strong aria-hidden="true">10.1.</strong> Perception Bus</a></li><li class="chapter-item "><a href="core-systems/ai/planning.html"><strong aria-hidden="true">10.2.</strong> Planning Layer</a></li><li class="chapter-item "><a href="core-systems/ai/tools.html"><strong aria-hidden="true">10.3.</strong> Tool Sandbox</a></li><li class="chapter-item "><a href="core-systems/ai/behavior-trees.html"><strong aria-hidden="true">10.4.</strong> Behavior Trees</a></li></ol></li><li class="chapter-item expanded "><a href="core-systems/physics.html"><strong aria-hidden="true">11.</strong> Physics</a></li><li class="chapter-item expanded "><a href="core-systems/rendering.html"><strong aria-hidden="true">12.</strong> Rendering</a></li><li class="chapter-item expanded "><a href="core-systems/audio.html"><strong aria-hidden="true">13.</strong> Audio</a></li><li class="chapter-item expanded "><a href="core-systems/navigation.html"><strong aria-hidden="true">14.</strong> Navigation</a></li><li class="chapter-item expanded "><a href="core-systems/input.html"><strong aria-hidden="true">15.</strong> Input System</a></li><li class="chapter-item expanded "><a href="core-systems/networking.html"><strong aria-hidden="true">16.</strong> Networking</a></li><li class="chapter-item expanded affix "><li class="part-title">Game Development</li><li class="chapter-item expanded "><a href="game-dev/first-game.html"><strong aria-hidden="true">17.</strong> Building Your First Game</a></li><li class="chapter-item expanded "><a href="game-dev/companions.html"><strong aria-hidden="true">18.</strong> AI Companions</a></li><li class="chapter-item expanded "><a href="game-dev/bosses.html"><strong aria-hidden="true">19.</strong> Adaptive Bosses</a></li><li class="chapter-item expanded "><a href="game-dev/crafting-combat.html"><strong aria-hidden="true">20.</strong> Crafting &amp; Combat</a></li><li class="chapter-item expanded "><a href="game-dev/dialogue.html"><strong aria-hidden="true">21.</strong> Dialogue Systems</a></li><li class="chapter-item expanded "><a href="game-dev/procedural.html"><strong aria-hidden="true">22.</strong> Procedural Content</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples &amp; Tutorials</li><li class="chapter-item expanded "><a href="examples/index.html"><strong aria-hidden="true">23.</strong> Working Examples</a></li><li class="chapter-item expanded "><a href="examples/hello-companion.html"><strong aria-hidden="true">24.</strong> Hello Companion</a></li><li class="chapter-item expanded "><a href="examples/adaptive-boss.html"><strong aria-hidden="true">25.</strong> Adaptive Boss</a></li><li class="chapter-item expanded "><a href="examples/physics-demo.html"><strong aria-hidden="true">26.</strong> Physics Demo</a></li><li class="chapter-item expanded "><a href="examples/navmesh-demo.html"><strong aria-hidden="true">27.</strong> Navmesh Demo</a></li><li class="chapter-item expanded "><a href="examples/audio-spatial.html"><strong aria-hidden="true">28.</strong> Audio Spatial</a></li><li class="chapter-item expanded "><a href="examples/troubleshooting.html"><strong aria-hidden="true">29.</strong> Troubleshooting Examples</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference Implementation</li><li class="chapter-item expanded "><a href="../../Games-VEILWEAVER/documentation/overview.html"><strong aria-hidden="true">30.</strong> Veilweaver Overview</a></li><li class="chapter-item expanded "><a href="../../Games-VEILWEAVER/documentation/mechanics.html"><strong aria-hidden="true">31.</strong> Game Mechanics</a></li><li class="chapter-item expanded "><a href="../../Games-VEILWEAVER/documentation/ai-integration.html"><strong aria-hidden="true">32.</strong> AI Integration</a></li><li class="chapter-item expanded "><a href="../../Games-VEILWEAVER/documentation/world-design.html"><strong aria-hidden="true">33.</strong> World Design</a></li><li class="chapter-item expanded affix "><li class="part-title">Engine Development</li><li class="chapter-item expanded "><a href="dev/contributing.html"><strong aria-hidden="true">34.</strong> Contributing Guide</a></li><li class="chapter-item expanded "><a href="dev/building.html"><strong aria-hidden="true">35.</strong> Building from Source</a></li><li class="chapter-item expanded "><a href="dev/testing.html"><strong aria-hidden="true">36.</strong> Testing</a></li><li class="chapter-item expanded "><a href="dev/code-style.html"><strong aria-hidden="true">37.</strong> Code Style</a></li><li class="chapter-item expanded "><a href="dev/new-features.html"><strong aria-hidden="true">38.</strong> Adding New Features</a></li><li class="chapter-item expanded "><a href="dev/performance.html"><strong aria-hidden="true">39.</strong> Performance Optimization</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="api/index.html"><strong aria-hidden="true">40.</strong> API Documentation</a></li><li class="chapter-item expanded "><a href="reference/crates.html"><strong aria-hidden="true">41.</strong> Crate Documentation</a></li><li class="chapter-item expanded "><a href="reference/configuration.html"><strong aria-hidden="true">42.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="reference/cli-tools.html"><strong aria-hidden="true">43.</strong> Command Line Tools</a></li><li class="chapter-item expanded "><a href="reference/platforms.html"><strong aria-hidden="true">44.</strong> Platform Support</a></li><li class="chapter-item expanded affix "><li class="part-title">Resources</li><li class="chapter-item expanded "><a href="resources/faq.html"><strong aria-hidden="true">45.</strong> FAQ</a></li><li class="chapter-item expanded "><a href="resources/performance.html"><strong aria-hidden="true">46.</strong> Performance Tips</a></li><li class="chapter-item expanded "><a href="resources/best-practices.html"><strong aria-hidden="true">47.</strong> Best Practices</a></li><li class="chapter-item expanded "><a href="resources/patterns.html"><strong aria-hidden="true">48.</strong> Common Patterns</a></li><li class="chapter-item expanded "><a href="resources/troubleshooting.html"><strong aria-hidden="true">49.</strong> Troubleshooting</a></li><li class="chapter-item expanded "><a href="resources/community.html"><strong aria-hidden="true">50.</strong> Community</a></li><li class="chapter-item expanded "><a href="../root-archive/roadmap.html"><strong aria-hidden="true">51.</strong> Roadmap</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
