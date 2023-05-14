#include "plugin.hpp"

struct PolyShuffle : Module {
    potential::PolyShuffle *inner = NULL;

    enum ParamId { PARAMS_LEN };
    enum InputId { INPUT_INPUT, SHUFFLE_TRIGGER_INPUT, INPUTS_LEN };
    enum OutputId { OUTPUT_OUTPUT, OUTPUTS_LEN };
    enum LightId { LIGHTS_LEN };

    PolyShuffle() {
        this->inner = potential::polyshuffle_new();

        config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
        configInput(INPUT_INPUT, "Polyphonic");
        configInput(SHUFFLE_TRIGGER_INPUT, "Shuffle trigger");
        configOutput(OUTPUT_OUTPUT, "Polyphonic");
    }

    ~PolyShuffle() { potential::polyshuffle_free(this->inner); }

    void process(const ProcessArgs &args) override {
        Input *poly_inputs = &inputs[INPUT_INPUT];
        Input *shuffle_trigger = &inputs[SHUFFLE_TRIGGER_INPUT];
        Output *poly_outputs = &outputs[OUTPUT_OUTPUT];

        int inputs_len = poly_inputs->getChannels();

        int outputs_len = potential::polyshuffle_process(
            this->inner, poly_inputs->getVoltages(), inputs_len,
            poly_outputs->getVoltages(), PORT_MAX_CHANNELS,
            shuffle_trigger->getVoltage());
        poly_outputs->setChannels(outputs_len);
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
