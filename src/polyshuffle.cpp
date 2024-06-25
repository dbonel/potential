#include "plugin.hpp"

struct PolyShuffle : Module {
    rustlib::PolyShuffle *inner = NULL;
    rustlib::ModuleConfigInfo *config_info = NULL;

    enum ParamId { PARAMS_LEN };
    enum InputId { INPUT_INPUT, SHUFFLE_TRIGGER_INPUT, INPUTS_LEN };
    enum OutputId { OUTPUT_OUTPUT, OUTPUTS_LEN };
    enum LightId { LIGHTS_LEN };

    PolyShuffle() {
        this->inner = rustlib::polyshuffle_new();
        this->config_info = this->inner->get_module_config_info();
        configure_from_info(this, this->config_info);
    }

    ~PolyShuffle() {
        rustlib::polyshuffle_free(this->inner);
        rustlib::module_config_free(this->config_info);
    }

    void process(const ProcessArgs &args) override {
        const rustlib::Port *inputs = ffi_port(&this->inputs[0]);
        rustlib::Port *outputs = ffi_port(&this->outputs[0]);

        this->inner->process_raw(inputs, outputs);
    }
};

struct PolyShuffleWidget : ModuleWidget {
    PolyShuffleWidget(PolyShuffle *module) {
        setModule(module);
        setPanel(
            createPanel(asset::plugin(pluginInstance, "res/PolyShuffle.svg")));

        addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
        addChild(
            createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH,
                                          RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 40.526)), module, PolyShuffle::INPUT_INPUT));
        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 59.362)), module,
            PolyShuffle::SHUFFLE_TRIGGER_INPUT));

        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 78.198)), module, PolyShuffle::OUTPUT_OUTPUT));
    }
};

Model *modelPolyShuffle =
    createModel<PolyShuffle, PolyShuffleWidget>("PolyShuffle");
